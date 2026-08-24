#ifndef LILYPAD_IKI_SENSOR_FACTORY_H
#define LILYPAD_IKI_SENSOR_FACTORY_H

#include "iki-sensor.h"
#include "standard-iki-sensor.h"
#include <memory>
#include "../lilypad-utils.h"

namespace fcitx {

    /**
     * @brief IkiSensorFactory - Khởi tạo Module Cảm biến IKI
     */
    class IkiSensorFactory {
      public:
        static std::unique_ptr<IIkiSensor> create_sensor(bool enabled = true, uint64_t min_ms = 10, uint64_t max_ms = 500) {
            LILYPAD_INFO("🔌 [IKI SENSOR FACTORY] Loaded StandardIkiSensor module (Enabled: " +
                         std::string(enabled ? "true" : "false") + ").");
            return std::make_unique<StandardIkiSensor>(enabled, min_ms, max_ms);
        }
    };

} // namespace fcitx

#endif // LILYPAD_IKI_SENSOR_FACTORY_H
