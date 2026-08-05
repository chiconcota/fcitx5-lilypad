#ifndef LILYPAD_SENSOR_FACTORY_H
#define LILYPAD_SENSOR_FACTORY_H

#include "ack-sensor.h"
#include "niri-sensor.h"
#include "generic-sensor.h"
#include <memory>
#include <cstdlib>
#include <string>
#include <fcitx-utils/log.h>

namespace fcitx {

    /**
     * @brief AckSensorFactory - Tự động phát hiện môi trường Compositor để nạp đúng Module Cảm biến
     */
    class AckSensorFactory {
      public:
        static std::unique_ptr<IAckSensor> create_sensor() {
            const char* desktop_env = std::getenv("XDG_CURRENT_DESKTOP");
            const char* wayland_disp = std::getenv("WAYLAND_DISPLAY");

            std::string desktop = desktop_env ? desktop_env : "";
            std::string disp = wayland_disp ? wayland_disp : "";

            if (desktop == "niri" || desktop == "Niri" || disp.find("niri") != std::string::npos) {
                LILYPAD_INFO("🔌 [SENSOR FACTORY] Compositor Niri detected. Loaded NiriAckSensor module.");
                return std::make_unique<NiriAckSensor>();
            }

            LILYPAD_INFO("🔌 [SENSOR FACTORY] Generic environment detected (" + desktop + "). Loaded GenericAckSensor fallback module.");
            return std::make_unique<GenericAckSensor>();
        }
    };

} // namespace fcitx

#endif // LILYPAD_SENSOR_FACTORY_H
