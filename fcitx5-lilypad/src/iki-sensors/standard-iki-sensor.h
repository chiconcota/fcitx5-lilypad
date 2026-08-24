#ifndef LILYPAD_STANDARD_IKI_SENSOR_H
#define LILYPAD_STANDARD_IKI_SENSOR_H

#include "iki-sensor.h"
#include <atomic>
#include <algorithm>
#include <fcitx-utils/log.h>

namespace fcitx {

    /**
     * @brief StandardIkiSensor - Module Cảm biến IKI tiêu chuẩn sử dụng thuật toán EMA Smoothing
     */
    class StandardIkiSensor : public IIkiSensor {
      public:
        StandardIkiSensor(bool enabled = true, uint64_t min_ms = 10, uint64_t max_ms = 500, uint64_t burst_threshold_ms = 35)
            : enabled_(enabled), min_ms_(min_ms), max_ms_(max_ms), burst_threshold_ms_(burst_threshold_ms) {}

        ~StandardIkiSensor() override = default;

        std::string get_name() const override {
            return "StandardIkiSensor";
        }

        void on_key_event(std::chrono::steady_clock::time_point now, bool is_synthetic) override {
            if (!enabled_ || is_synthetic) {
                return;
            }

            if (last_physical_key_time_.time_since_epoch().count() > 0) {
                auto delta = std::chrono::duration_cast<std::chrono::milliseconds>(now - last_physical_key_time_).count();

                // Lọc bỏ khoảng nghỉ idle giữa các từ (> 1000ms) hoặc xung nảy phím micro-glitch (< 5ms)
                if (delta >= 5 && delta <= 1000) {
                    uint64_t clamped_delta = static_cast<uint64_t>(std::clamp<int64_t>(delta, min_ms_, max_ms_));
                    current_iki_ms_.store(clamped_delta, std::memory_order_release);

                    // Thuật toán EMA (Exponential Moving Average): 35% mẫu mới + 65% lịch sử
                    uint64_t prev_ema = iki_ema_ms_.load(std::memory_order_relaxed);
                    uint64_t new_ema = static_cast<uint64_t>(0.35 * static_cast<double>(clamped_delta) + 0.65 * static_cast<double>(prev_ema));
                    new_ema = static_cast<uint64_t>(std::clamp<int64_t>(new_ema, min_ms_, max_ms_));
                    iki_ema_ms_.store(new_ema, std::memory_order_release);

                    LILYPAD_INFO("⌨️ [IKI SENSOR] Key delta: " + std::to_string(delta) + "ms (Clamped: " +
                                 std::to_string(clamped_delta) + "ms) | EMA IKI: " + std::to_string(new_ema) +
                                 "ms (Burst: " + (is_burst_typing() ? "YES" : "NO") + ")");
                }
            }

            last_physical_key_time_ = now;
        }

        void reset() override {
            last_physical_key_time_ = {};
        }

        uint64_t get_current_iki_ms() const override {
            return current_iki_ms_.load(std::memory_order_acquire);
        }

        uint64_t get_ema_iki_ms() const override {
            return iki_ema_ms_.load(std::memory_order_acquire);
        }

        bool is_burst_typing() const override {
            return iki_ema_ms_.load(std::memory_order_acquire) <= burst_threshold_ms_;
        }

        void set_enabled(bool enabled) {
            enabled_ = enabled;
        }

      private:
        bool                                  enabled_ = true;
        uint64_t                              min_ms_ = 10;
        uint64_t                              max_ms_ = 500;
        uint64_t                              burst_threshold_ms_ = 35;
        std::chrono::steady_clock::time_point last_physical_key_time_{};
        std::atomic<uint64_t>                 current_iki_ms_{150};
        std::atomic<uint64_t>                 iki_ema_ms_{150};
    };

} // namespace fcitx

#endif // LILYPAD_STANDARD_IKI_SENSOR_H
