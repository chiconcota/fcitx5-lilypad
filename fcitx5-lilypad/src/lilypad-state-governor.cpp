/*
 * SPDX-FileCopyrightText: 2022-2022 CSSlayer <wengxt@gmail.com>
 * SPDX-FileCopyrightText: 2025-2026 Lý Tất Thành
 * SPDX-FileCopyrightText: 2026 Antigravity
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/**
 * @file lilypad-state-governor.cpp
 * @brief Dynamic Adaptive Governor, Sequencer pacing, and Micro-replacement Engine.
 */

#include "lilypad-state.h"
#include "lilypad-engine.h"
#include "lilypad-candidates.h"
#include "lilypad-utils.h"
#include "lilypad.h"

#include <cstddef>
#include <fcitx-utils/log.h>
#include <fcitx-utils/utf8.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputpanel.h>
#include <fcitx/userinterface.h>

#include <algorithm>
#include <chrono>
#include <string>

#include "ack-sensors/sensor-factory.h"

namespace fcitx {

    void LilypadState::executeCommitSequence(bool from_safety_timer, uint64_t swallow_duration_us) {
        if (commit_triggered_) {
            return;
        }
        commit_triggered_ = true;

        if (barrier_safety_timer_) {
            barrier_safety_timer_.reset();
        }
        if (focus_settled_timer_) {
            focus_settled_timer_.reset();
        }
        sequencer_.clear_barrier();

        // Multi-tier Adaptive Lag Confirmation & Telemetry Decoupling:
        // If rescued by safety timer, distinguish REAL LAG (focus bounce or queue pressure) from FALSE ALARM (smooth tab).
        bool confirmed_lag = false;
        if (from_safety_timer) {
            confirmed_lag = has_recent_focus_bounce_ || !buffered_keys_.empty();
            if (confirmed_lag) {
                last_swallow_duration_us_ = swallow_duration_us;
                LILYPAD_INFO("🎯 [DYNAMIC GOVERNOR] Safety rescue confirmed REAL LAG (focus_bounce=" +
                             std::to_string(has_recent_focus_bounce_) + ", queue=" +
                             std::to_string(buffered_keys_.size()) + ") -> Preserving lag telemetry (" +
                             std::to_string(swallow_duration_us) + "us)");
            } else {
                LILYPAD_INFO("🎯 [DYNAMIC GOVERNOR] Safety rescue on SMOOTH TAB (no focus bounce, empty queue) -> Decoupled telemetry to protect smooth pacing");
            }
        } else {
            last_swallow_duration_us_ = swallow_duration_us;
        }

        MicroStep step;
        if (sequencer_.poll_next_step(step) && step.type == MicroStepType::CommitString) {
            auto& eventLoop = engine_->instance()->eventLoop();
            auto  now_time  = ::fcitx::now(CLOCK_MONOTONIC);
            int bsCount = std::max(1, original_bs_count_);

            if (!from_safety_timer && sequencer_.sensor()) {
                sequencer_.sensor()->on_swallow_measured(bsCount, swallow_duration_us);
            }

            uint64_t settling_delay_us = 0;
            if (from_safety_timer) {
                // If confirmed lag on heavy DOM (TikTok / AFFiNE), give settling breathing room
                // to prevent commit string from colliding with pending DOM reflow.
                // Standard safe floor matching 1 VSync frame for Chromium.
                if (wa_chromium_flag) {
                    settling_delay_us = confirmed_lag ? 22000 : 18000;
                } else {
                    settling_delay_us = confirmed_lag ? 10000 : 100;
                }
            } else {
                uint64_t current_iki = (iki_sensor_ && *engine_->config().enableIkiAdaptive) ? iki_sensor_->get_ema_iki_ms() : 0;
                uint64_t micro_delay_us = sequencer_.sensor() ? sequencer_.sensor()->get_micro_delay_us(bsCount, current_iki) : (6000 + static_cast<uint64_t>(bsCount * 4000));
                settling_delay_us = calculateDynamicSettlingDelayUs(micro_delay_us, swallow_duration_us);
            }

            sequencer_.set_settling_delay(settling_delay_us);

            LILYPAD_INFO("⚡ [SEQUENCER SETTLING DELAY] Delay: " + std::to_string(settling_delay_us) +
                         "us (from_safety_timer=" + std::to_string(from_safety_timer) +
                         ", swallow=" + std::to_string(swallow_duration_us) +
                         "us, bsCount=" + std::to_string(bsCount) +
                         ", wa_chromium=" + std::to_string(wa_chromium_flag) + ")");

            auto timeout_time = now_time + settling_delay_us;
            std::string commitStr = step.text;
            uint32_t serial = step.serial;
            commit_timer_ = eventLoop.addTimeEvent(CLOCK_MONOTONIC, timeout_time, 1, [this, commitStr, serial](EventSourceTime*, uint64_t) {
                if (watchdog_timer_) {
                    watchdog_timer_.reset();
                }
                ic_->commitString(commitStr);
                LILYPAD_INFO("Commit (Sequence Mode): " + commitStr);
                sequencer_.receive_ack(serial);

                // Phase 8.3: Dynamic Post-Commit Settling Window (Per-Tab Adaptive Control)
                uint64_t settle_delay_us = calculateDynamicPostSettlingUs();

                LILYPAD_INFO("⏳ [POST-COMMIT SETTLING] Holding is_deleting_ for " + std::to_string(settle_delay_us) + "us (wa_chromium=" + std::to_string(wa_chromium_flag) + ")");
                auto& loop = engine_->instance()->eventLoop();
                auto  t    = ::fcitx::now(CLOCK_MONOTONIC) + settle_delay_us;
                settle_timer_ = loop.addTimeEvent(CLOCK_MONOTONIC, t, 1, [this](EventSourceTime*, uint64_t) {
                    if (g_active_deleting_state.load(std::memory_order_acquire) == static_cast<void*>(this)) {
                        g_active_deleting_state.store(nullptr, std::memory_order_release);
                    }
                    is_deleting_.store(false, std::memory_order_release);
                    g_last_deleting_state.store(static_cast<void*>(this), std::memory_order_release);
                    g_last_deleting_end_time_ms.store(now_ms(), std::memory_order_release);

                    if (!buffered_keys_.empty()) {
                        is_replaying_ = true;
                        LILYPAD_INFO("Replaying " + std::to_string(buffered_keys_.size()) + " buffered keys after settling window");
                        replayBufferedKeys();
                    } else {
                        is_replaying_ = false;
                    }
                    return false;
                });
                return false;
            });
        }
    }

    void LilypadState::performReplacement(const std::string& deletedPart, const std::string& addedPart) {
        LILYPAD_INFO("Perform replacement: " + deletedPart + " -> " + addedPart); //NOLINT
        if (settle_timer_) {
            settle_timer_.reset();
        }
        if (disarm_timer_) {
            disarm_timer_.reset();
        }
        if (replay_timer_) {
            replay_timer_.reset();
        }
        is_replaying_ = false;
        if (commit_timer_) {
            commit_timer_.reset();
        }
        if (focus_settled_timer_) {
            focus_settled_timer_.reset();
        }

        if (realMode == LilypadMode::Sequence) {
            // Micro-replacement:
            // Delete only the minimal suffix `deletedPart` using exact utf8 codepoint length,
            // capped by max word length oldPreBuffer_ to guarantee word boundary safety.
            int bsCount = static_cast<int>(utf8::length(deletedPart));
            if (!oldPreBuffer_.empty() && deletedPart != " " && deletedPart != "-") {
                int maxBs = static_cast<int>(utf8::length(oldPreBuffer_));
                if (bsCount > maxBs) {
                    LILYPAD_WARN("Capping backspaces from " + std::to_string(bsCount) + " to max word length " + std::to_string(maxBs));
                    bsCount = maxBs;
                }
            }

            // Phase 8.4: Hardware Barrier Key KEY_PAUSE Integration
            // Send exactly bsCount backspaces for deletion. The uinput daemon will automatically
            // append 1 atomic KEY_PAUSE event right after the backspaces as the non-destructive barrier sentinel.
            uint32_t serial = sequencer_.next_serial();
            if (bsCount > 0) {
                MicroStep bsStep;
                bsStep.type = MicroStepType::EmitBackspace;
                bsStep.count = bsCount;
                bsStep.serial = serial;
                sequencer_.push_action(bsStep);
            }
            MicroStep commitStep;
            commitStep.type = MicroStepType::CommitString;
            commitStep.text = addedPart;
            commitStep.serial = serial;
            sequencer_.push_action(commitStep);

            // Pop EmitBackspace step so barrier is active and CommitString step is at front of queue
            if (bsCount > 0) {
                MicroStep dummyBs;
                sequencer_.poll_next_step(dummyBs);
            }

            pending_commit_string_   = addedPart;
            original_bs_count_       = bsCount;
            expected_backspaces_     = bsCount;
            current_backspace_count_ = 0;
            commit_triggered_        = false;
            g_active_deleting_state.store(static_cast<void*>(this), std::memory_order_release);
            is_deleting_.store(true, std::memory_order_release);

            auto& eventLoop = engine_->instance()->eventLoop();
            auto  now_time  = ::fcitx::now(CLOCK_MONOTONIC);

            // Phase 8.5: Hardware Sentinel KEY_PAUSE Safety & Telemetry.
            // In Linux evdev & Wayland FIFO sockets, hardware sentinel KEY_PAUSE arrives in 2-5ms.
            // If heavy DOM reflow or compositor delays KEY_PAUSE beyond 120ms, safety rescue timer
            // commits the text safely before the 300ms watchdog purge, guaranteeing zero swallowed characters.
            if (barrier_safety_timer_) {
                barrier_safety_timer_.reset();
            }
            uint64_t lag_timeout_us = 120000; // 120ms safety rescue
            auto lag_timeout_time = now_time + lag_timeout_us;
            barrier_safety_timer_ = eventLoop.addTimeEvent(CLOCK_MONOTONIC, lag_timeout_time, 1, [this, serial](EventSourceTime*, uint64_t) {
                if (commit_triggered_ || !is_deleting_.load(std::memory_order_acquire)) {
                    return false;
                }
                auto now = std::chrono::steady_clock::now();
                uint64_t swallow_us = (uinput_send_time_.time_since_epoch().count() > 0)
                    ? static_cast<uint64_t>(std::chrono::duration_cast<std::chrono::microseconds>(now - uinput_send_time_).count())
                    : 120000;
                LILYPAD_INFO("🛡️ [SAFETY TIMER RESCUE] 120ms elapsed without KEY_PAUSE sentinel. Rescuing commit safely! [Serial #" + std::to_string(serial) + "]");
                executeCommitSequence(true, swallow_us);
                return false;
            });

            // Arm Hard Timeout Watchdog (Unified 300ms for all apps)
            if (watchdog_timer_) {
                watchdog_timer_.reset();
            }
            uint64_t hard_timeout_us = 300000;
            auto  hard_timeout_time = now_time + hard_timeout_us;
            watchdog_timer_ = eventLoop.addTimeEvent(CLOCK_MONOTONIC, hard_timeout_time, 1000, [this, hard_timeout_us](EventSourceTime*, uint64_t) {
                if (is_deleting_.load(std::memory_order_acquire)) {
                    LILYPAD_WARN("⏱️ [WATCHDOG TIMEOUT] " + std::to_string(hard_timeout_us / 1000) + "ms elapsed with is_deleting_ still active. Triggering emergency purge!");
                    purgeContextEmergency();
                }
                return false;
            });

            uinput_send_time_ = std::chrono::steady_clock::now();
            send_backspace_uinput(bsCount);
            LILYPAD_INFO("Send " + std::to_string(bsCount) + " backspaces (Sequence Mode - Hardware Barrier Key KEY_PAUSE Sentinel)");
            return;
        }

        current_backspace_count_ = 0;
        pending_commit_string_   = addedPart;
        expected_backspaces_     = static_cast<int>(utf8::length(deletedPart));
        if (realMode != LilypadMode::Minecraft) {
            ++expected_backspaces_;
            const auto& surrounding = ic_->surroundingText();
            if (isAutofillCertain(surrounding)) {
                ++expected_backspaces_;
            }
        }
        g_active_deleting_state.store(static_cast<void*>(this), std::memory_order_release);
        is_deleting_.store(true, std::memory_order_release);
        send_backspace_uinput(expected_backspaces_);
        LILYPAD_INFO("Send " + std::to_string(expected_backspaces_) + " backspaces");
    }

    void LilypadState::recordFocusBounce() {
        last_focus_bounce_time_ = std::chrono::steady_clock::now();
        has_recent_focus_bounce_ = true;
        LILYPAD_INFO("🎯 [DYNAMIC GOVERNOR] Recorded Focus Bounce event for context");
    }

    bool LilypadState::isTabLagging() const {
        if (!wa_chromium_flag) {
            return false;
        }
        auto now = std::chrono::steady_clock::now();

        // Signal 1: Key queue pressure in RAM
        if (!buffered_keys_.empty()) {
            auto key_wait_ms = std::chrono::duration_cast<std::chrono::milliseconds>(
                now - buffered_keys_.front().enter_time
            ).count();
            if (key_wait_ms >= 15 || buffered_keys_.size() >= 2) {
                return true;
            }
        }

        // Signal 2: Measured swallow duration of uinput backspace loop
        // Normal swallow on light tab is ~2ms - 6ms. If >= 12ms, DOM/renderer is lagging.
        if (last_swallow_duration_us_ >= 12000) {
            return true;
        }

        // Signal 3: Measured App ACK from sequencer sensor
        if (sequencer_.sensor() && sequencer_.sensor()->get_last_measured_ack_ms() >= 30) {
            return true;
        }

        // Signal 4: Recent focus bounce within last 3 seconds
        if (has_recent_focus_bounce_) {
            auto bounce_elapsed_ms = std::chrono::duration_cast<std::chrono::milliseconds>(
                now - last_focus_bounce_time_
            ).count();
            if (bounce_elapsed_ms < 3000) {
                return true;
            }
        }

        return false;
    }

    uint64_t LilypadState::calculateDynamicSettlingDelayUs(uint64_t micro_delay_us, uint64_t swallow_duration_us) {
        if (!wa_chromium_flag) {
            return 1500; // Zero-latency floor for native / terminal apps
        }

        bool lagging = isTabLagging();
        if (!lagging) {
            // Smooth Chromium Tab (Facebook, Docs, Antigravity IDE, Discord, etc.)
            // Standard safe 1 VSync frame breathing room (18ms - 22ms) matching Lotus defaultCommitInterval (20ms)
            uint64_t delay_us = std::clamp<uint64_t>(micro_delay_us, 18000, 22000);
            LILYPAD_INFO("⚡ [DYNAMIC GOVERNOR] Smooth Tab detected -> Pre-commit Settling Delay: " +
                         std::to_string(delay_us) + "us");
            return delay_us;
        }

        // Heavy-DOM / Lagging Tab (TikTok 25K comments, AFFiNE Canvas)
        // Adaptive scaling based on measured swallow, safely bounded between 20ms and 35ms
        uint64_t delay_us = std::clamp<uint64_t>(
            (swallow_duration_us > 0) ? (swallow_duration_us * 8 / 10) : 20000,
            20000,
            35000
        );

        LILYPAD_INFO("⏳ [DYNAMIC GOVERNOR] Heavy DOM / Lagging Tab detected -> Dynamic Settling Delay: " +
                     std::to_string(delay_us) + "us (swallow=" + std::to_string(swallow_duration_us) +
                     "us, queue=" + std::to_string(buffered_keys_.size()) + ")");
        return delay_us;
    }

    uint64_t LilypadState::calculateDynamicPostSettlingUs() const {
        if (!wa_chromium_flag) {
            return 300; // Ultra-fast 0.3ms for Terminal / Qt / Native apps
        }

        // Fast-path for queued keys: Replay after safe 5ms micro-breather (matching Lotus backspace interval)
        // so fast subsequent keystrokes (like 'n' in 'thương') replay reliably without DOM collision
        if (!buffered_keys_.empty()) {
            return 5000;
        }

        bool lagging = isTabLagging();
        uint64_t settle_delay_us = 18000; // Baseline 18ms (1 full frame @ 60Hz + safety margin)

        if (!lagging) {
            settle_delay_us = 18000;
        } else {
            // Dynamic breathing room capped safely at 35ms (max 2 frames) on heavy DOM
            uint64_t lag_expansion = (last_swallow_duration_us_ > 0) ? (last_swallow_duration_us_ * 5 / 10) : 0;
            settle_delay_us = std::clamp<uint64_t>(18000 + lag_expansion, 18000, 35000);
        }

        LILYPAD_INFO("⏳ [DYNAMIC GOVERNOR] Post-Commit Settling Window: " + std::to_string(settle_delay_us) +
                     "us (lagging=" + std::to_string(lagging) + ", swallow=" + std::to_string(last_swallow_duration_us_) +
                     "us, queue=" + std::to_string(buffered_keys_.size()) + ")");
        return settle_delay_us;
    }

    void LilypadState::replayBufferedKeys() {
        LILYPAD_INFO("Starting replay buffered keys (Batch Commit Pipeline)");
        if (buffered_keys_.empty()) {
            is_replaying_ = false;
            if (replay_timer_) {
                replay_timer_.reset();
            }
            return;
        }
        is_replaying_ = true;
        auto keys = std::move(buffered_keys_);
        buffered_keys_.clear();

        std::string batch_commit;

        for (size_t i = 0; i < keys.size(); ++i) {
            auto        sym     = static_cast<KeySym>(keys[i].sym);
            uint32_t    state   = keys[i].state;
            std::string keyUtf8 = Key::keySymToUTF8(sym);
            if (keyUtf8.empty()) {
                continue;
            }

            bool processed = EngineProcessKeyEvent(lilypadEngine_.handle(), sym, state) != 0U;

            auto commitF = UniqueCPtr<char>(EnginePullCommit(lilypadEngine_.handle()));
            if (commitF && (*commitF.get() != 0)) {
                std::string commitStr = commitF.get();
                std::string deletedPart;
                std::string addedPart;
                compareAndSplitStrings(oldPreBuffer_, commitStr, deletedPart, addedPart);

                if (!deletedPart.empty()) {
                    for (size_t j = i + 1; j < keys.size(); ++j) {
                        if (buffered_keys_.size() < MAX_BUFFERED_KEYS) {
                            buffered_keys_.push_back(keys[j]);
                        }
                    }
                    if (!batch_commit.empty()) {
                        ic_->commitString(batch_commit);
                        LILYPAD_INFO("Commit (Batch Replay Pre): " + batch_commit);
                        batch_commit.clear();
                    }
                    is_replaying_ = false;
                    performReplacement(deletedPart, addedPart);
                    hasHistory_ = false;
                    ResetEngine(lilypadEngine_.handle());
                    oldPreBuffer_.clear();
                    return;
                }
                if (!addedPart.empty()) {
                    batch_commit += addedPart;
                }

                hasHistory_ = false;
                ResetEngine(lilypadEngine_.handle());
                oldPreBuffer_.clear();
                continue;
            }

            if (!processed) {
                batch_commit += keyUtf8;
                continue;
            }

            hasHistory_ = true;
            realtextLen.fetch_add(1, std::memory_order_acq_rel);

            UniqueCPtr<char> preeditC(EnginePullPreedit(lilypadEngine_.handle()));
            std::string      preeditStr = (preeditC && (*preeditC.get() != 0)) ? preeditC.get() : "";

            std::string      deletedPart;
            std::string      addedPart;
            if (compareAndSplitStrings(oldPreBuffer_, preeditStr, deletedPart, addedPart) != 0) {
                if (deletedPart.empty()) {
                    if (!addedPart.empty()) {
                        batch_commit += addedPart;
                        oldPreBuffer_ = preeditStr;
                    }
                } else {
                    if (uinput_client_fd_ < 0) {
                        batch_commit += keyUtf8;
                        continue;
                    }

                    for (size_t j = i + 1; j < keys.size(); ++j) {
                        if (buffered_keys_.size() < MAX_BUFFERED_KEYS) {
                            buffered_keys_.push_back(keys[j]);
                        }
                    }
                    if (!batch_commit.empty()) {
                        ic_->commitString(batch_commit);
                        LILYPAD_INFO("Commit (Batch Replay Pre): " + batch_commit);
                        batch_commit.clear();
                    }
                    is_replaying_ = false;
                    performReplacement(deletedPart, addedPart);
                    oldPreBuffer_ = preeditStr;
                    return;
                }
            }
        }
        if (!batch_commit.empty()) {
            ic_->commitString(batch_commit);
            LILYPAD_INFO("Commit (Batch Replay Final): " + batch_commit);
        }
        is_replaying_ = false;
        if (replay_timer_) {
            replay_timer_.reset();
        }
        LILYPAD_INFO("Replay buffered keys done");
    }

    void LilypadState::purgeContextEmergency() {
        LILYPAD_WARN("⚠️ [HARD TIMEOUT EMERGENCY] App freeze / Timeout (>300ms). Purging state & flushing buffered keys.");
        if (watchdog_timer_) {
            watchdog_timer_.reset();
        }
        if (barrier_safety_timer_) {
            barrier_safety_timer_.reset();
        }
        if (focus_settled_timer_) {
            focus_settled_timer_.reset();
        }
        if (commit_timer_) {
            commit_timer_.reset();
        }
        if (settle_timer_) {
            settle_timer_.reset();
        }
        if (disarm_timer_) {
            disarm_timer_.reset();
        }
        if (replay_timer_) {
            replay_timer_.reset();
        }
        is_replaying_ = false;
        commit_triggered_ = false;
        if (g_active_deleting_state.load(std::memory_order_acquire) == static_cast<void*>(this)) {
            g_active_deleting_state.store(nullptr, std::memory_order_release);
        }
        is_deleting_.store(false, std::memory_order_release);
        send_disarm_tap_uinput();
        dedup_buffer_.clear();
        expected_backspaces_     = 0;
        current_backspace_count_ = 0;
        pending_commit_string_.clear();
        hasHistory_ = false;
        ResetEngine(lilypadEngine_.handle());
        oldPreBuffer_.clear();
        sequencer_.clear();

        if (!buffered_keys_.empty()) {
            LILYPAD_INFO("Flushing " + std::to_string(buffered_keys_.size()) + " buffered keys as raw input");
            for (const auto& k : buffered_keys_) {
                ic_->forwardKey(Key(static_cast<KeySym>(k.sym), KeyStates(k.state)));
            }
            buffered_keys_.clear();
        }
    }

    void LilypadState::scheduleFocusSettledCommit(uint64_t debounce_us) {
        if (!is_deleting_.load(std::memory_order_acquire) || commit_triggered_) {
            return;
        }
        auto& loop = engine_->instance()->eventLoop();
        if (focus_settled_timer_) {
            focus_settled_timer_.reset();
        }
        auto now_time = ::fcitx::now(CLOCK_MONOTONIC);
        auto t = now_time + debounce_us;
        focus_settled_timer_ = loop.addTimeEvent(CLOCK_MONOTONIC, t, 1, [this](EventSourceTime*, uint64_t) {
            if (!is_deleting_.load(std::memory_order_acquire) || commit_triggered_) {
                return false;
            }
            auto now = std::chrono::steady_clock::now();
            uint64_t swallow_us = (uinput_send_time_.time_since_epoch().count() > 0)
                ? static_cast<uint64_t>(std::chrono::duration_cast<std::chrono::microseconds>(now - uinput_send_time_).count())
                : 35000;
            LILYPAD_INFO("🎯 [FOCUS SETTLED] Chromium focus bounce settled after " +
                         std::to_string(swallow_us / 1000) + "ms. Executing commit safely!");
            executeCommitSequence(true, swallow_us);
            return false;
        });
    }

} // namespace fcitx

