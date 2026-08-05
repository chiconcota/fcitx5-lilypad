#ifndef LILYPAD_SEQUENCER_H
#define LILYPAD_SEQUENCER_H

#include <chrono>
#include <cstdint>
#include <deque>
#include <string>
#include <atomic>
#include <fcitx-utils/log.h>

namespace fcitx {

    enum class MicroStepType : std::uint8_t {
        ForwardKey,
        EmitBackspace,
        CommitString,
        SetPreedit,
        PassthroughNonVietnameseKey
    };

    struct MicroStep {
        MicroStepType type;
        int           count = 0;
        std::string   text;
        uint32_t      keycode = 0;
        uint32_t      state   = 0;
        uint32_t      serial  = 0; ///< Serial ID for Order Enforcement
    };

    enum class BarrierState : std::uint8_t {
        Ready,
        WaitingMicroDelay,
        WaitingForAck
    };

    struct SequencerConfig {
        uint64_t min_delay_ms       = 5;   ///< 5ms micro-delay
        uint64_t max_ack_timeout_ms = 250; ///< 250ms safety timeout for Wayland Frame ACK
    };

    /**
     * @brief Sequencer Layer - State-Aware MicroStep Orchestration, Serial Tagging & Adaptive App Speed Engine
     */
    class Sequencer {
      public:
        Sequencer();
        ~Sequencer() = default;

        void     clear();
        uint32_t next_serial();
        uint32_t active_serial() const {
            return active_serial_.load();
        }

        void push_action(const MicroStep& step);
        bool poll_next_step(MicroStep& out_step);
        bool should_swallow_backspace(uint32_t serial = 0);
        void receive_ack(uint32_t serial);
        void set_waiting_ack();
        void clear_barrier() {
            barrier_ = BarrierState::Ready;
        }

        size_t queue_size() const {
            return queue_.size();
        }
        int expected_swallow_backspaces() const {
            return expected_swallow_backspaces_.load();
        }

        /// Calculates dynamic adaptive delay based on app ACK response time
        uint64_t calculate_adaptive_delay_ms(uint64_t measured_ack_ms);

      private:
        std::deque<MicroStep>                 queue_;
        BarrierState                          barrier_ = BarrierState::Ready;
        std::chrono::steady_clock::time_point barrier_start_time_;
        std::chrono::steady_clock::time_point barrier_target_time_;

        std::atomic<uint32_t>                 serial_counter_{1};
        std::atomic<uint32_t>                 active_serial_{0};

        std::atomic<int>                      expected_swallow_backspaces_{0};
        SequencerConfig                       config_;
        uint64_t                              last_measured_ack_ms_ = 5;
    };

} // namespace fcitx

#endif // LILYPAD_SEQUENCER_H
