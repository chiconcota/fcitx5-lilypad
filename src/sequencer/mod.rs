//! Sequencer Layer - State-Aware MicroStep Orchestration & ACK Barrier Engine
//!
//! Controls the deterministic execution queue for IME actions.
//! Prevents async race conditions, duplicated text, and swallowed backspaces.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MicroStep {
    ForwardKey { keycode: u32, state: u32 },
    EmitBackspace { count: usize },
    CommitString { text: String },
    SetPreedit { text: String },
    PassthroughNonVietnameseKey { keycode: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarrierState {
    Ready,
    WaitingMicroDelay(Instant),
    WaitingForAck { serial: u32, sent_at: Instant },
}

#[derive(Debug)]
pub struct SequencerConfig {
    pub micro_delay_ms: u64,
    pub ack_timeout_ms: u64,
}

impl Default for SequencerConfig {
    fn default() -> Self {
        Self {
            micro_delay_ms: 1,
            ack_timeout_ms: 5, // Smart 5ms ACK timeout for ultra-responsive typing
        }
    }
}

#[derive(Debug)]
pub struct Sequencer {
    pub queue: VecDeque<MicroStep>,
    pub barrier: BarrierState,
    pub expected_swallow_backspaces: usize,
    pub config: SequencerConfig,
}

impl Sequencer {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            barrier: BarrierState::Ready,
            expected_swallow_backspaces: 0,
            config: SequencerConfig::default(),
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.barrier = BarrierState::Ready;
        self.expected_swallow_backspaces = 0;
    }

    pub fn push_action(&mut self, step: MicroStep) {
        // Queue Deduplication / Coalescing for spammed non-Vietnamese keys (e.g. Enter bounce)
        if let MicroStep::PassthroughNonVietnameseKey { keycode } = &step {
            if let Some(MicroStep::PassthroughNonVietnameseKey { keycode: last_key }) = self.queue.back() {
                if last_key == keycode {
                    info!(target: "vnlilypad::sequencer", keycode = keycode, "🛡️ [SEQUENCER DEDUP] Coalesced duplicate non-Vietnamese key event in queue");
                    return;
                }
            }
        }

        info!(target: "vnlilypad::sequencer", step = ?step, "📥 [SEQUENCER PUSH] Action queued");
        if let MicroStep::EmitBackspace { count } = step {
            self.expected_swallow_backspaces += count;
            info!(target: "vnlilypad::sequencer", count = count, total = self.expected_swallow_backspaces, "🛡️ [TOKEN COUNT] Added expected uinput backspace swallow token");
        }
        self.queue.push_back(step);
    }

    pub fn receive_ack(&mut self, serial: u32) {
        if let BarrierState::WaitingForAck { serial: expected_serial, sent_at } = self.barrier {
            if serial >= expected_serial {
                let elapsed = sent_at.elapsed();
                info!(target: "vnlilypad::sequencer", serial = serial, elapsed_ms = ?elapsed, "✅ [ACK BARRIER RELEASED] Compositor responded with ACK");
                self.barrier = BarrierState::Ready;
            }
        }
    }

    pub fn poll_next_step(&mut self) -> Option<MicroStep> {
        // Check barrier timeout or micro delay completion
        match self.barrier {
            BarrierState::WaitingMicroDelay(target_time) => {
                if Instant::now() >= target_time {
                    info!(target: "vnlilypad::sequencer", "⏱️ [MICRO-DELAY DONE] 1ms backspace delay completed, releasing barrier");
                    self.barrier = BarrierState::Ready;
                } else {
                    return None;
                }
            }
            BarrierState::WaitingForAck { sent_at, serial } => {
                if sent_at.elapsed() >= Duration::from_millis(self.config.ack_timeout_ms) {
                    info!(target: "vnlilypad::sequencer", serial = serial, "⏱️ [ACK BARRIER DONE] Smart 5ms timeout reached, unblocking barrier");
                    self.barrier = BarrierState::Ready;
                } else {
                    return None;
                }
            }
            BarrierState::Ready => {}
        }

        let step = self.queue.pop_front()?;
        info!(target: "vnlilypad::sequencer", step = ?step, "⚡ [SEQUENCER EXECUTE] Dispatching MicroStep");

        match &step {
            MicroStep::EmitBackspace { .. } => {
                let delay = Duration::from_millis(self.config.micro_delay_ms);
                self.barrier = BarrierState::WaitingMicroDelay(Instant::now() + delay);
            }
            MicroStep::CommitString { .. } => {
                // Set barrier waiting for Wayland compositor ACK
            }
            _ => {}
        }

        Some(step)
    }

    pub fn set_waiting_ack(&mut self, serial: u32) {
        info!(target: "vnlilypad::sequencer", serial = serial, "🔒 [ACK BARRIER SET] Waiting for Compositor ACK Done event");
        self.barrier = BarrierState::WaitingForAck {
            serial,
            sent_at: Instant::now(),
        };
    }

    pub fn should_swallow_backspace(&mut self) -> bool {
        if self.expected_swallow_backspaces > 0 {
            self.expected_swallow_backspaces -= 1;
            info!(target: "vnlilypad::sequencer", remaining = self.expected_swallow_backspaces, "🛡️ [SWALLOW BACKSPACE] Consumed uinput backspace token cleanly");
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequencer_queue_and_token_swallow() {
        let mut seq = Sequencer::new();
        seq.push_action(MicroStep::EmitBackspace { count: 2 });
        assert_eq!(seq.expected_swallow_backspaces, 2);

        assert!(seq.should_swallow_backspace());
        assert_eq!(seq.expected_swallow_backspaces, 1);
        assert!(seq.should_swallow_backspace());
        assert_eq!(seq.expected_swallow_backspaces, 0);
        assert!(!seq.should_swallow_backspace());
    }

    #[test]
    fn test_sequencer_microstep_poll() {
        let mut seq = Sequencer::new();
        seq.push_action(MicroStep::CommitString { text: "á".into() });

        let step = seq.poll_next_step();
        assert_eq!(step, Some(MicroStep::CommitString { text: "á".into() }));
    }
}
