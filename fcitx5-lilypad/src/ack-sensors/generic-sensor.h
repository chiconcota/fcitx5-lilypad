#ifndef LILYPAD_GENERIC_SENSOR_H
#define LILYPAD_GENERIC_SENSOR_H

#include "ack-sensor.h"
#include <atomic>
#include <algorithm>
#include <fcitx-utils/log.h>

    namespace fcitx {

    /**
     * @brief GenericAckSensor - Module Cảm biến ACK dự phòng Fallback cho mọi Compositor
     */
    class GenericAckSensor : public IAckSensor {
      public:
        GenericAckSensor()           = default;
        ~GenericAckSensor() override = default;

        std::string get_name() const override {
            return "GenericAckSensor";
        }

        void on_transaction_start(uint32_t serial) override {
            active_serial_.store(serial, std::memory_order_release);
            start_time_ = std::chrono::steady_clock::now();
            has_start_time_.store(true, std::memory_order_release);
        }

        void on_ack_received(uint32_t serial) override {
            if (has_start_time_.load(std::memory_order_acquire) && serial >= active_serial_.load(std::memory_order_acquire)) {
                has_start_time_.store(false, std::memory_order_release);
                auto     elapsed  = std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now() - start_time_).count();
                uint64_t prev     = last_measured_ack_ms_.load(std::memory_order_acquire);
                uint64_t measured = static_cast<uint64_t>(std::max<int64_t>(1, elapsed));
                // EMA Machine Learning Style Adaptive Control: 35% measured + 65% history
                uint64_t adaptive = std::clamp<uint64_t>(static_cast<uint64_t>(0.35 * measured + 0.65 * prev), min_delay_ms_, max_ack_timeout_ms_);
                last_measured_ack_ms_.store(adaptive, std::memory_order_release);
                LILYPAD_INFO("📊 [GENERIC SENSOR ACK] Serial #" + std::to_string(serial) + " elapsed=" + std::to_string(elapsed) +
                             "ms -> EMA Adaptive Delay: " + std::to_string(adaptive) + "ms");
            }
        }

        uint64_t get_micro_delay_us(int bsCount, uint64_t iki_ms = 0) const override {
            int count = std::max(1, bsCount);
            if (iki_ms == 0) {
                // Cold Start Baseline: Mức trần an toàn >50ms bảo đảm 100% chữ đầu tiên không bao giờ lỗi
                return 35000 + static_cast<uint64_t>(count * 15000); // 1bs=50ms, 2bs=65ms, 3bs=80ms
            }

            // Chuẩn hóa Min-Max (Feature Scaling): IKI in [35ms, 150ms] -> t in [0.0, 1.0]
            double t = std::clamp((static_cast<double>(iki_ms) - 35.0) / (150.0 - 35.0), 0.0, 1.0);

            // Base settling delay: 1.5ms (Burst) -> 15.0ms (Safe Web DOM)
            double base_us = 1500.0 + t * (15000.0 - 1500.0);

            // Thời gian tiêu thụ mỗi phím xóa kết hợp Cảm biến App ACK đo thực tế:
            uint64_t app_ack_us = last_measured_ack_ms_.load(std::memory_order_acquire) * 1000;
            double min_per_bs_us = 800.0 + t * (18000.0 - 800.0);
            double per_bs_us = std::max(min_per_bs_us, static_cast<double>(app_ack_us));

            uint64_t total_us = static_cast<uint64_t>(base_us + static_cast<double>(count) * per_bs_us);
            return std::max<uint64_t>(1500, total_us);
        }

        uint64_t get_last_measured_ack_ms() const override {
            return last_measured_ack_ms_.load(std::memory_order_acquire);
        }

        uint64_t get_max_ack_timeout_ms() const override {
            return max_ack_timeout_ms_;
        }

      private:
        std::atomic<uint32_t>                 active_serial_{0};
        std::atomic<uint64_t>                 last_measured_ack_ms_{5};
        std::atomic<bool>                     has_start_time_{false};
        std::chrono::steady_clock::time_point start_time_;
        uint64_t                              min_delay_ms_       = 5;
        uint64_t                              max_ack_timeout_ms_ = 250;
    };

} // namespace fcitx

#endif // LILYPAD_GENERIC_SENSOR_H
