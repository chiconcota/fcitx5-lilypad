#ifndef LILYPAD_IKI_SENSOR_H
#define LILYPAD_IKI_SENSOR_H

#include <chrono>
#include <cstdint>
#include <string>
#include "../lilypad-utils.h"

namespace fcitx {

    /**
     * @brief Interface IIkiSensor - Abstract Base Class cho các Module Cảm biến Đo tốc độ gõ (Inter-Keystroke Interval)
     */
    class IIkiSensor {
      public:
        virtual ~IIkiSensor() = default;

        /// Tên định danh của Module Cảm biến IKI
        virtual std::string get_name() const = 0;

        /// Bắt sự kiện phím nhấn vật lý để đo delta_t
        virtual void on_key_event(std::chrono::steady_clock::time_point now, bool is_synthetic) = 0;

        /// Reset mốc thời gian khi đổi cửa sổ hoặc xóa bộ đệm
        virtual void reset() = 0;

        /// Lấy delta_t của phím gõ gần nhất (millisecond)
        virtual uint64_t get_current_iki_ms() const = 0;

        /// Lấy giá trị IKI làm mịn EMA (millisecond)
        virtual uint64_t get_ema_iki_ms() const = 0;

        /// Kiểm tra xem người dùng có đang gõ lướt siêu tốc (Burst Typing) hay không
        virtual bool is_burst_typing() const = 0;
    };

} // namespace fcitx

#endif // LILYPAD_IKI_SENSOR_H
