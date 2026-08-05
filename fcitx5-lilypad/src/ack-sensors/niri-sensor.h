#ifndef LILYPAD_NIRI_SENSOR_H
#define LILYPAD_NIRI_SENSOR_H

#include "ack-sensor.h"
#include <atomic>
#include <algorithm>
#include <fcitx-utils/log.h>

namespace fcitx {

    /**
     * @brief NiriAckSensor - Module Cảm biến ACK dành riêng cho Niri / Sway / Wayland v2 Compositor
     */
    class NiriAckSensor : public IAckSensor {
      public:
        NiriAckSensor() = default;
        ~NiriAckSensor() override = default;

        std::string get_name() const override {
            return "NiriAckSensor";
        }

        void on_transaction_start(uint32_t serial) override {
            active_serial_.store(serial, std::memory_order_release);
            start_time_ = std::chrono::steady_clock::now();
            has_start_time_.store(true, std::memory_order_release);
        }

        void on_ack_received(uint32_t serial) override {
            if (has_start_time_.load(std::memory_order_acquire) && serial >= active_serial_.load(std::memory_order_acquire)) {
                has_start_time_.store(false, std::memory_order_release);
                auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
                    std::chrono::steady_clock::now() - start_time_
                ).count();
                uint64_t measured = static_cast<uint64_t>(std::max<int64_t>(1, elapsed));
                // Instant Direct Tracking: Bám đuổi 100% độ trễ thực tế của App (100ms -> 150ms -> 5ms)
                uint64_t adaptive = std::clamp<uint64_t>(measured, min_delay_ms_, max_ack_timeout_ms_);
                last_measured_ack_ms_.store(adaptive, std::memory_order_release);
                LILYPAD_INFO("📊 [NIRI SENSOR ACK] Serial #" + std::to_string(serial) + " elapsed=" + std::to_string(elapsed) + "ms -> Instant Adaptive Delay: " + std::to_string(adaptive) + "ms");
            }
        }

        uint64_t get_micro_delay_us(int bsCount) const override {
            int count = std::max(1, bsCount);
            return 6000 + static_cast<uint64_t>(count * 4000); // 1bs=10ms, 2bs=14ms, 3bs=18ms
        }

        uint64_t get_last_measured_ack_ms() const override {
            return last_measured_ack_ms_.load(std::memory_order_acquire);
        }

        uint64_t get_max_ack_timeout_ms() const override {
            return max_ack_timeout_ms_;
        }

      private:
        std::atomic<uint32_t> active_serial_{0};
        std::atomic<uint64_t> last_measured_ack_ms_{5};
        std::atomic<bool>     has_start_time_{false};
        std::chrono::steady_clock::time_point start_time_;
        uint64_t min_delay_ms_ = 5;
        uint64_t max_ack_timeout_ms_ = 250;
    };

} // namespace fcitx

#endif // LILYPAD_NIRI_SENSOR_H
