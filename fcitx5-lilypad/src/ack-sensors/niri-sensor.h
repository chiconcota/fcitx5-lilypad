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
                uint64_t prev = last_measured_ack_ms_.load(std::memory_order_acquire);
                uint64_t measured = static_cast<uint64_t>(std::max<int64_t>(1, elapsed));
                // EMA Machine Learning Style Adaptive Control: 35% measured + 65% history
                uint64_t adaptive = std::clamp<uint64_t>(
                    static_cast<uint64_t>(0.35 * measured + 0.65 * prev),
                    min_delay_ms_,
                    max_ack_timeout_ms_
                );
                last_measured_ack_ms_.store(adaptive, std::memory_order_release);
                LILYPAD_INFO("📊 [NIRI SENSOR ACK] Serial #" + std::to_string(serial) + " elapsed=" + std::to_string(elapsed) + "ms -> EMA Adaptive Delay: " + std::to_string(adaptive) + "ms");
            }
        }

        uint64_t get_micro_delay_us(int bsCount, uint64_t iki_ms = 0) const override {
            int count = std::max(1, bsCount);
            if (iki_ms == 0) {
                return 6000 + static_cast<uint64_t>(count * 4000); // Fallback: 1bs=10ms, 2bs=14ms, 3bs=18ms
            }

            // Co giãn trơn liên tục (Continuous Scale Factor) theo EMA IKI:
            // IKI = 25ms -> alpha = 0.16 -> 1bs: 1.6ms, 2bs: 2.2ms
            // IKI = 80ms -> alpha = 0.53 -> 1bs: 5.3ms, 2bs: 7.4ms
            // IKI >= 150ms -> alpha = 1.0 -> 1bs: 10ms, 2bs: 14ms
            double alpha = std::clamp(static_cast<double>(iki_ms) / 150.0, 0.15, 1.0);

            if (count == 1) {
                // Fast-Path cho thao tác xóa 1 phím (thay đổi dấu thanh/nguyên âm: a -> á, e -> ê)
                return std::max<uint64_t>(1000, static_cast<uint64_t>(10000 * alpha));
            }

            // Multi-char suffix replacement (hoang -> hoàng: 3bs, nghieng -> nghiêng: 4bs)
            uint64_t raw_delay = 6000 + static_cast<uint64_t>(count * 4000);
            uint64_t scaled_delay = static_cast<uint64_t>(raw_delay * alpha);
            uint64_t min_floor = 1000 + static_cast<uint64_t>(count * 500); // 2bs=2.0ms, 3bs=2.5ms floor
            return std::max<uint64_t>(min_floor, scaled_delay);
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
