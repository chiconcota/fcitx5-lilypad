#ifndef LILYPAD_ACK_SENSOR_H
#define LILYPAD_ACK_SENSOR_H

#include <chrono>
#include <cstdint>
#include <string>
#include "../lilypad-utils.h"

namespace fcitx {

    /**
     * @brief Interface IAckSensor - Abstract Base Class cho các Module Cảm biến ACK
     */
    class IAckSensor {
      public:
        virtual ~IAckSensor() = default;

        /// Tên định danh của Module Cảm biến (e.g. "NiriAckSensor", "GenericAckSensor")
        virtual std::string get_name() const = 0;

        /// Gọi khi bắt đầu một giao dịch thay thế từ (xuất phát đồng hồ đo)
        virtual void on_transaction_start(uint32_t serial) = 0;

        /// Gọi khi nhận tín hiệu ACK hoặc hoàn thành giao dịch (dừng đồng hồ đo)
        virtual void on_ack_received(uint32_t serial) = 0;

        /// Lấy thời gian ngắt nhịp vi mô (microsecond) dựa trên số phím xóa và nhịp gõ IKI
        virtual uint64_t get_micro_delay_us(int bsCount, uint64_t iki_ms = 0) const = 0;

        /// Lấy thời gian trễ thích ứng vừa đo được (millisecond)
        virtual uint64_t get_last_measured_ack_ms() const = 0;

        /// Lấy trần Safety Timeout cho Compositor này (millisecond)
        virtual uint64_t get_max_ack_timeout_ms() const = 0;
    };

} // namespace fcitx

#endif // LILYPAD_ACK_SENSOR_H
