# ĐẶC TẢ KIẾN TRÚC & NGUYÊN TẮC KỸ THUẬT: MODULAR ACK SENSOR (`ack-sensors/`)

@module: `fcitx5-lilypad/src/ack-sensors/` | @architecture: Strategy Pattern + Universal Wayland IME Protocol | @target: Developers & Contributors

---

## 📌 1. NGUYÊN TẮC VẠN NĂNG (UNIVERSAL WAYLAND IME PROTOCOL PRINCIPLE)

> **KẾT LUẬN KIẾN TRÚC CỐT LÕI:** 
> **Bộ gõ KHÔNG CẦN phải viết Sensor riêng cho từng Linux Distro hay từng Compositor!**
> 
> Bất kể người dùng chạy distro nào (Arch, Debian, Ubuntu, Fedora, NixOS, Void...) hay Compositor nào (Niri, Hyprland, Sway, KDE, GNOME), **miễn là hệ thống hỗ trợ chuẩn giao thức `zwp_input_method_v1` hoặc `v2`**, cơ chế **Evdev Uinput Passthrough + Token Swallow** đều hoạt động vạn năng (Universal) 100%!

---

## 🛠️ 2. VAI TRÒ CỦA `GenericAckSensor` - MODULE SENSOR VẠN NĂNG

Mã nguồn `fcitx5-lilypad/src/ack-sensors/` được thiết kế tối giản với 2 Sensor chính:

### 2.1 `GenericAckSensor` (`generic-sensor.h`) - SENSOR VẠN NĂNG CHO MỌI DISTRO & COMPOSITOR
- **Xử lý tất cả các Distro & Compositor** (Hyprland, Sway, KDE Plasma, GNOME Mutter, river, Wayfire...):
  - Tự động nạp qua `AckSensorFactory` cho mọi môi trường không phải Niri.
  - Công thức Micro-Delay Pacing nới rộng an toàn vạn năng:
    $$\text{micro\_delay\_us} = 8000 + \text{bsCount} \times 5000\,\mu\text{s}$$
    *(1 backspace = 13ms, 2 backspaces = 18ms, 3 backspaces = 23ms)*.

### 2.2 `NiriAckSensor` (`niri-sensor.h`) - VI ĐIỀU CHỈNH OPTIMIZED HOÃN NHỊP
- **Chỉ là một bản Tinh chỉnh Thông số (Micro-tuning Parameter Optimization)** dành riêng cho Niri Compositor do tác giả đang trực tiếp kiểm thử:
  - Công thức Micro-Delay Pacing tối ưu hóa cho Niri:
    $$\text{micro\_delay\_us} = 6000 + \text{bsCount} \times 4000\,\mu\text{s}$$
    *(1 backspace = 10ms, 2 backspaces = 14ms, 3 backspaces = 18ms)*.

### 2.3 `AckSensorFactory` (`sensor-factory.h`)
- Kiểm tra `$XDG_CURRENT_DESKTOP`:
  - Nếu là `"niri"` $\rightarrow$ Nạp `NiriAckSensor` (bản vi điều chỉnh).
  - Tất cả các trường hợp còn lại $\rightarrow$ Nạp `GenericAckSensor` (bản vạn năng).

---

## 🛤️ 3. HÀNH TRÌNH 5 BƯỚC VẠN NĂNG TRÊN LINUX WAYLAND INPUT SUBSYSTEM

```text
  ┌────────────────────────────────────────────────────────────────────────┐
  │ 1. KERNEL UINPUT DAEMON                                                │
  │    - fcitx5-lilypad-server ghi struct input_event vào /dev/uinput     │
  │    - Kernel tạo sự kiện evdev tại /dev/input/eventX                    │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼ (Mở & Đọc file bằng libinput)
  ┌────────────────────────────────────────────────────────────────────────┐
  │ 2. WINDOW COMPOSITOR (BẤT KỲ DISTRO / COMPOSITOR NÀO)                 │
  │    - Compositor mở /dev/input/eventX bằng `libinput`                   │
  │    - Định tuyến phím sang Fcitx5 qua `zwp_input_method_v1/v2`          │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼ (Wayland Protocol: zwp_input_method_v1/v2)
  ┌────────────────────────────────────────────────────────────────────────┐
  │ 3. BỘ GÕ FCITX5 (LILYPAD)                                             │
  │    - LilypadState::handleUInputKeyPress() nhận được phím từ Compositor │
  │    - Sequencer trừ 1 token: expected_swallow_backspaces_.fetch_sub(1) │
  │    - Trả `return false` chuyển tiếp phím xóa vào App                    │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼ (Wayland Protocol: wl_keyboard.key / X11)
  ┌────────────────────────────────────────────────────────────────────────┐
  │ 4. ỨNG DỤNG BÊN NGOÀI (Chrome / Messenger / Electron 39 / AFFiNE)     │
  │    - App nhận phím xóa KEY_BACKSPACE và tiến hành xóa ký tự trong DOM  │
  └────────────────────────────────────────────────────────────────────────┘
```

---

## 📊 4. BẰNG CHỨNG THỰC NGHIỆM TRỰC TIẾP TỪ LOG MESSENGER

Dữ liệu log thực tế khi gõ tốc độ cao trên ứng dụng **Messenger (Electron Web App)**:

```text
I2026-08-06 16:36:06.643572 ✅ [SERIAL ACK RELEASED] Serial #7  elapsed=95ms  -> Dynamic Barrier: 96ms
I2026-08-06 16:36:07.241823 ✅ [SERIAL ACK RELEASED] Serial #8  elapsed=69ms  -> Dynamic Barrier: 70ms
I2026-08-06 16:36:08.115729 ✅ [SERIAL ACK RELEASED] Serial #9  elapsed=63ms  -> Dynamic Barrier: 64ms
I2026-08-06 16:36:09.203777 ✅ [SERIAL ACK RELEASED] Serial #11 elapsed=148ms -> Dynamic Barrier: 149ms (Thay thế 3 phím: ưng -> ứng)
I2026-08-06 16:36:12.741843 ✅ [SERIAL ACK RELEASED] Serial #14 elapsed=25ms  -> Dynamic Barrier: 26ms (Thay thế nhanh: u -> ụ)
I2026-08-06 16:36:17.991798 ✅ [SERIAL ACK RELEASED] Serial #18 elapsed=130ms -> Dynamic Barrier: 131ms (Messenger lag DOM: o -> ọ)
I2026-08-06 16:36:26.909993 ✅ [SERIAL ACK RELEASED] Serial #22 elapsed=151ms -> Dynamic Barrier: 152ms (Messenger lag khựng DOM)
```

---

## 📍 5. BỐN ĐOẠN CODE CỐT LÕI VÀ VỊ TRÍ CHI TIẾT TRONG MÃ NGUỒN C++

- **Mốc $T_1$:** [fcitx5-lilypad/src/lilypad-sequencer.cpp:L52-L59](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.cpp#L52-L59) (`Sequencer::set_waiting_ack()`).
- **Nuốt Token Xóa & Lập Timer:** [fcitx5-lilypad/src/lilypad-state.cpp:L446-L475](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-state.cpp#L446-L475) (`LilypadState::handleUInputKeyPress()`).
- **Mốc $T_2$ & Tính `elapsed`:** [fcitx5-lilypad/src/lilypad-sequencer.cpp:L61-L75](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.cpp#L61-L75) (`Sequencer::receive_ack()`).
- **Tính Dynamic Barrier:** [fcitx5-lilypad/src/lilypad-sequencer.cpp:L44-L50](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.cpp#L44-L50) (`Sequencer::calculate_adaptive_delay_ms()`).
