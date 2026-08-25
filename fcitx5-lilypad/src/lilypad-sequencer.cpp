#include "lilypad-sequencer.h"
#include "lilypad-utils.h"
#include <algorithm>

namespace fcitx {

    Sequencer::Sequencer() : expected_swallow_backspaces_(0) {}

    void Sequencer::clear() {
        queue_.clear();
        barrier_ = BarrierState::Ready;
        expected_swallow_backspaces_.store(0);
    }

    uint32_t Sequencer::next_serial() {
        uint32_t s = serial_counter_.fetch_add(1, std::memory_order_acq_rel);
        active_serial_.store(s, std::memory_order_release);
        return s;
    }

    void Sequencer::push_action(const MicroStep& step) {
        // Queue Deduplication / Coalescing for spammed non-Vietnamese keys (e.g. Enter bounce)
        if (step.type == MicroStepType::PassthroughNonVietnameseKey) {
            if (!queue_.empty() && queue_.back().type == MicroStepType::PassthroughNonVietnameseKey) {
                if (queue_.back().keycode == step.keycode) {
                    LILYPAD_INFO("🛡️ [SEQUENCER DEDUP] Coalesced duplicate non-Vietnamese key event in queue");
                    return;
                }
            }
        }

        LILYPAD_INFO("📥 [SEQUENCER PUSH] Action queued [Serial #" + std::to_string(step.serial) + "]: type=" + std::to_string(static_cast<int>(step.type)) + " text=" + step.text +
                   " count=" + std::to_string(step.count));

        if (step.type == MicroStepType::EmitBackspace) {
            expected_swallow_backspaces_.fetch_add(step.count, std::memory_order_acq_rel);
            LILYPAD_INFO("🛡️ [TOKEN COUNT] Added expected uinput backspace swallow token [Serial #" + std::to_string(step.serial) + "]: count=" + std::to_string(step.count) +
                       " total=" + std::to_string(expected_swallow_backspaces_.load()));
        }

        queue_.push_back(step);
    }

    uint64_t Sequencer::calculate_adaptive_delay_ms(uint64_t measured_ack_ms) {
        // Adaptive formula: min_delay_ms (5ms) <= adaptive <= max_ack_timeout_ms (250ms)
        uint64_t adaptive = std::clamp(measured_ack_ms + 1, config_.min_delay_ms, config_.max_ack_timeout_ms);
        last_measured_ack_ms_ = adaptive;
        LILYPAD_INFO("📊 [ADAPTIVE DELAY] Measured App ACK: " + std::to_string(measured_ack_ms) + "ms -> Dynamic Barrier: " + std::to_string(adaptive) + "ms");
        return adaptive;
    }

    void Sequencer::set_waiting_ack() {
        barrier_            = BarrierState::WaitingForAck;
        barrier_start_time_ = std::chrono::steady_clock::now();
        if (sensor_) {
            sensor_->on_transaction_start(active_serial_.load());
        }
        LILYPAD_INFO("🛡️ [SEQUENCER BARRIER] Set WaitingForAck (Sensor: " + (sensor_ ? sensor_->get_name() : "Native") + ") for Serial #" + std::to_string(active_serial_.load()));
    }

    void Sequencer::set_app_lag_holding() {
        barrier_ = BarrierState::AppLagHolding;
        LILYPAD_INFO("🛡️ [SEQUENCER BARRIER] Set AppLagHolding for Serial #" + std::to_string(active_serial_.load()));
    }

    int64_t Sequencer::elapsed_since_barrier_start_ms() const {
        if (barrier_ == BarrierState::Ready) {
            return 0;
        }
        return std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now() - barrier_start_time_).count();
    }

    uint64_t Sequencer::calculate_soft_timeout_ms(uint64_t iki_ms) const {
        uint64_t app_ack = sensor_ ? sensor_->get_last_measured_ack_ms() : last_measured_ack_ms_;
        uint64_t expected = app_ack + 5; // Expected roundtrip with micro delay
        uint64_t iki_ref = (iki_ms > 0) ? iki_ms : 150;
        uint64_t soft = std::clamp<uint64_t>(
            std::max(expected * 2, std::min(iki_ref, expected + 30)),
            35,
            120
        );
        return soft;
    }

    bool Sequencer::is_soft_timeout(uint64_t iki_ms) const {
        if (barrier_ == BarrierState::Ready) {
            return false;
        }
        return elapsed_since_barrier_start_ms() >= static_cast<int64_t>(calculate_soft_timeout_ms(iki_ms));
    }

    bool Sequencer::is_hard_timeout() const {
        if (barrier_ == BarrierState::Ready) {
            return false;
        }
        return elapsed_since_barrier_start_ms() >= static_cast<int64_t>(config_.max_ack_timeout_ms);
    }

    void Sequencer::receive_ack(uint32_t serial) {
        if (barrier_ == BarrierState::WaitingForAck || barrier_ == BarrierState::AppLagHolding) {
            if (serial >= active_serial_.load(std::memory_order_acquire)) {
                auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now() - barrier_start_time_).count();
                LILYPAD_INFO("✅ [SERIAL ACK RELEASED] Compositor responded for Serial #" + std::to_string(serial) + " elapsed=" + std::to_string(elapsed) + "ms");
                calculate_adaptive_delay_ms(elapsed);
                if (sensor_) {
                    sensor_->on_ack_received(serial);
                }
                barrier_ = BarrierState::Ready;
            } else {
                LILYPAD_INFO("🛡️ [SERIAL STALE DISCARD] Discarded out-of-order ACK Serial #" + std::to_string(serial) + " (active: #" + std::to_string(active_serial_.load()) + ")");
            }
        }
    }

    bool Sequencer::poll_next_step(MicroStep& out_step) {
        auto now = std::chrono::steady_clock::now();

        // Check barrier timeout or micro delay completion
        if (barrier_ == BarrierState::WaitingMicroDelay) {
            if (now >= barrier_target_time_) {
                LILYPAD_INFO("⏱️ [MICRO-DELAY DONE] Adaptive delay completed, releasing barrier");
                barrier_ = BarrierState::Ready;
            } else {
                return false;
            }
        } else if (barrier_ == BarrierState::WaitingForAck || barrier_ == BarrierState::AppLagHolding) {
            auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - barrier_start_time_).count();
            if (elapsed >= static_cast<int64_t>(config_.max_ack_timeout_ms)) {
                LILYPAD_INFO("⏱️ [SAFETY TIMEOUT] " + std::to_string(config_.max_ack_timeout_ms) + "ms limit reached without ACK signal. Unblocking Serial #" +
                           std::to_string(active_serial_.load()));
                barrier_ = BarrierState::Ready;
            } else {
                return false;
            }
        }

        while (!queue_.empty() && queue_.front().serial < active_serial_.load(std::memory_order_acquire)) {
            LILYPAD_INFO("🛡️ [SERIAL STALE DISCARD] Discarding outdated MicroStep [Serial #" +
                       std::to_string(queue_.front().serial) + "] for active Serial #" +
                       std::to_string(active_serial_.load()));
            queue_.pop_front();
        }

        if (queue_.empty()) {
            return false;
        }

        out_step = queue_.front();
        queue_.pop_front();

        LILYPAD_INFO("⚡ [SEQUENCER EXECUTE] Dispatching MicroStep [Serial #" + std::to_string(out_step.serial) + "]: type=" + std::to_string(static_cast<int>(out_step.type)) +
                   " text=" + out_step.text);

        if (out_step.type == MicroStepType::EmitBackspace) {
            uint64_t dynamic_delay_ms = sensor_ ? sensor_->get_last_measured_ack_ms() : last_measured_ack_ms_;
            barrier_             = BarrierState::WaitingMicroDelay;
            barrier_start_time_  = now;
            barrier_target_time_ = now + std::chrono::milliseconds(dynamic_delay_ms);
        } else if (out_step.type == MicroStepType::CommitString) {
            barrier_            = BarrierState::WaitingForAck;
            barrier_start_time_ = now;
        }

        return true;
    }

    bool Sequencer::should_swallow_backspace(uint32_t serial) {
        if (serial > 0 && serial < active_serial_.load(std::memory_order_acquire)) {
            LILYPAD_INFO("🛡️ [SERIAL STALE DISCARD] Ignored backspace from old Serial #" + std::to_string(serial));
            return false;
        }
        if (expected_swallow_backspaces_.load(std::memory_order_acquire) > 0) {
            expected_swallow_backspaces_.fetch_sub(1, std::memory_order_acq_rel);
            LILYPAD_INFO("🛡️ [SWALLOW BACKSPACE] Consumed uinput backspace token [Serial #" + std::to_string(active_serial_.load()) +
                       "]. Remaining: " + std::to_string(expected_swallow_backspaces_.load()));
            return true;
        }
        return false;
    }

} // namespace fcitx
