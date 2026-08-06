# VNLILYPAD LOTUS DECISION LOG (NHẬT KÝ QUYẾT ĐỊNH KIẾN TRÚC)

> **Kiến trúc Chuẩn:** Fcitx5 Lilypad Sequencer Architecture (`v2.2.0-modular-sensor`)
> **Nguyên tắc Đối soát:** Chỉ lưu trữ các Quyết định Kỹ thuật đang thực tế vận hành 100% trong mã nguồn C++ của `fcitx5-lilypad`.
> **Lưu trữ thử nghiệm đã gỡ bỏ [XOÁ - KHÔNG ÁP DỤNG]:** Xem chi tiết tại [archive/deprecated-decisions.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.vnlilypadlotus-ai/2-memory/archive/deprecated-decisions.md).

---

## 🎯 1. HỆ THỐNG CẢM BIẾN ADAPTIVE ACK & DYNAMIC LATENCY CONTROL

### [2026-08-05] Quyết định 013: Modular IAckSensor Architecture & Universal Wayland Protocol
- **Bối cảnh:** Các Compositor (Niri, Sway, Hyprland, KWin, Mutter) phát tín hiệu Wayland Frame ACK khác nhau, việc viết gộp vào Sequencer Core làm mã nguồn bị phình to và không đo được độ trễ thực tế.
- **Quyết định:**
  1. Tách lớp cảm biến thành các Module C++ cắm/rút (`IAckSensor`, `NiriAckSensor`, `GenericAckSensor`, `AckSensorFactory`).
  2. Tự động kiểm tra `$XDG_CURRENT_DESKTOP` để nạp đúng Module cảm biến phù hợp.
  3. `GenericAckSensor` đóng vai trò cảm biến Vạn năng (Universal) xử lý 100% các Distro & Compositor hỗ trợ `zwp_input_method_v1/v2`.
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/ack-sensors/` (`ack-sensor.h`, `niri-sensor.h`, `generic-sensor.h`, `sensor-factory.h`).

### [2026-08-05] Quyết định 010: Dynamic Adaptive ACK Sensor & Safety Timeout (250ms Cap)
- **Bối cảnh:** Khi ứng dụng Web/Electron (Messenger, Facebook Post) bị lag/jank DOM, rào chắn ACK cần tự động mở rộng để tạo khoảng thở an toàn cho phím gõ tiếp theo.
- **Quyết định:**
  1. Bắt đầu bấm giờ $T_1$ (`start_time_ = steady_clock::now()`) khi phím được nạp vào Sequencer.
  2. Dừng bấm giờ $T_2$ và đo `elapsed` thực tế khi token xóa được nuốt sạch và commit string nổ.
  3. Kích hoạt `calculate_adaptive_delay_ms(elapsed)` nạp `elapsed` vào công thức `clamp` để tính toán Dynamic Barrier cho phím tiếp theo.
  4. Đặt rào chắn Safety Timeout trần **250ms** trong `lilypad-sequencer.h/.cpp`. Nếu app bị đóng băng quá 250ms, Sequencer tự động xả rào chắn tránh treo phím.
- **Mã nguồn thực thi:** [fcitx5-lilypad/src/lilypad-sequencer.cpp:L61-L75](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.cpp#L61-L75).

---

## ⚡ 2. BỘ NÃO ĐIỀU PHỐI SEQUENCER LAYER & QUẢN LÝ HÀNG ĐỢI

### [2026-08-05] Quyết định 014: Optimized Batch Replay Protocol (0.1ms Character vs 3ms Space Micro-gap)
- **Bối cảnh:** Khi tay gõ siêu tốc trong lúc rào chắn đang bật, các phím đệm `buffered_keys_` bị hoãn $15\text{ms}$ cho từng phím con trong vòng lặp đệ quy, gây dồn tích độ trễ lên tới 4.5 giây (`Typing so fast, add key to queue`).
- **Quyết định:**
  1. Phân biệt loại phím đệm khi xả hàng đợi: Phím ký tự thường (`a, b, c...`) xả tức thì $0.1\text{ms}$ (Batch Flush) bẻ gãy bẫy đệ quy.
  2. Riêng phím CÁCH (`Space`): Giữ nhịp ngắt vi mô $3\text{ms}$ để tách gói IPC an toàn với Chromium/Facebook.
- **Mã nguồn thực thi:** [fcitx5-lilypad/src/lilypad-state.cpp:L463-L473](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-state.cpp#L463-L473).

### [2026-08-05] Quyết định 011: Stale Serial Microstep Pruning (`serial < active_serial_`)
- **Bối cảnh:** Khi tay gõ nhanh liên tiếp, các vi bước (`MicroStep`) của phím cũ còn tồn đọng ở đầu hàng đợi `queue_` có thể nổ chen ngang vào phím mới gây ra lỗi lặp rác chữ (`mminimln`, `choa`).
- **Quyết định:**
  1. Mỗi giao dịch mới được gán một Serial ID nguyên tử tăng dần (`active_serial_`).
  2. Hàm `poll_next_step()` chạy vòng lặp `while (queue_.front().serial < active_serial_)` để phát hiện và vứt bỏ (`pop_front()`) toàn bộ các vi bước cũ hết hạn.
- **Mã nguồn thực thi:** [fcitx5-lilypad/src/lilypad-sequencer.cpp:L99-L104](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.cpp#L99-L104).

---

## 🛠️ 3. HẠ TẦNG KERNEL UINPUT SERVER & XÓA VI MÔ

### [2026-08-05] Quyết định 012: Proportional Backspace Micro-delay & Micro-replacement (`deletedPart`)
- **Bối cảnh:** Việc xóa toàn bộ từ rồi gõ lại gây ra độ trễ lớn và làm giật con trỏ trên Web DOM.
- **Quyết định:**
  1. So sánh chuỗi cũ và mới qua `compareAndSplitStrings` để chỉ xóa phần hậu tố tối thiểu (`deletedPart`).
  2. Giới hạn trần số phím xóa `bsCount` bằng độ dài từ cũ (`maxBs = utf8::length(oldPreBuffer_)`) bảo vệ ranh giới từ.
  3. Áp dụng micro-delay tỷ lệ thuận với số phím xóa: `micro_delay_us = 6000 + bsCount * 4000` (1bs=10ms, 2bs=14ms, 3bs=18ms).
- **Mã nguồn thực thi:** [fcitx5-lilypad/src/lilypad-state.cpp:L519-L532](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-state.cpp#L519-L532).

### [2026-07-28] Quyết định 014 (Gốc 079): Pure Kernel Uinput Backspace Emission
- **Bối cảnh:** Việc sử dụng API `deleteSurroundingText()` gây xung đột dữ liệu và nhảy con trỏ trên các ứng dụng Electron và Web Editors.
- **Quyết định:**
  1. Nghiêm cấm 100% `deleteSurroundingText()` và `Preedit` trong Sequence Mode.
  2. 100% phím xóa được phát qua `/dev/uinput` server daemon dưới dạng mảng 4 sự kiện nguyên tử `ev[4]` (Press + SYN_REPORT + Release + SYN_REPORT) trong 1 lệnh `write()` duy nhất.
- **Mã nguồn thực thi:** [fcitx5-lilypad/server/lilypad-server.cpp:L72-L85](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/server/lilypad-server.cpp#L72-L85).

---

## 🚀 4. TÍCH HỢP FCITX5 FRAMEWORK & SYSTEMD USER SERVICE

### [2026-08-04] Quyết định 008: Chế Độ Gõ `Sequence` & Wayland IPC Keyboard Grab
- **Bối cảnh:** Cần thêm chế độ gõ chuyên biệt tích hợp Sequencer Layer trên giao diện cấu hình GUI và backend C++.
- **Quyết định:**
  1. Thêm `LilypadMode::Sequence` (Enum ID #9) vào `lilypad-config.h` và `lilypad-state.cpp`.
  2. Kích hoạt Wayland IPC Keyboard Grab (`enable_keyboard_grab = true`) đón phím mượt mà.
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/lilypad-config.h`, `lilypad-state.cpp`.

### [2026-08-04] Quyết định 007: Thương Hiệu Độc Lập `fcitx5-lilypad` & Systemd Template Service
- **Bối cảnh:** Tách độc lập bộ gõ thành gói `fcitx5-lilypad` (`liblilypad.so`), bảo tồn `fcitx5-lotus-main` làm reference backup.
- **Quyết định:**
  1. Tạo Systemd Template Unit Service `fcitx5-lilypad-server@.service` tự động kích hoạt theo `$USER` hệ thống (`sudo systemctl enable --now fcitx5-lilypad-server@$USER.service`).
  2. Bổ sung parse UID (`-u username|uid`) trong `lilypad-server.cpp` và xác thực an toàn IPC `SO_PEERCRED` (`cred.uid == expected_uid`).
- **Mã nguồn thực thi:** [fcitx5-lilypad/server/lilypad-server.cpp:L179-L197](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/server/lilypad-server.cpp#L179-L197), [misc/fcitx5-lilypad-server@.service](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/misc/fcitx5-lilypad-server@.service).

---

## 🛡️ 5. CHÍNH SÁCH ỨNG DỤNG ĐẶC THÙ (AFFiNE / ELECTRON 39)

### [2026-08-05] Quyết định 044: AFFiNE Spurious Focus Analysis & Zero-Regression Rollback Policy
- **Bối cảnh:** AFFiNE (Canvas/Shadow DOM Editor) phát tín hiệu `InputContextFocusIn` / `activate` ngầm 10ms sau mỗi lần `commitString()`. Thử nghiệm can thiệp code C++ riêng cho AFFiNE đã làm ảnh hưởng (regression) đến các ứng dụng khác.
- **Quyết định:**
  1. Thực hiện **Zero-Regression Rollback**: Không viết bất kỳ dòng code hack C++ riêng nào cho AFFiNE bên trong bộ gõ, giữ mã nguồn C++ sạch 100%.
  2. Hướng dẫn người dùng cấu hình cờ Electron Wayland IME trong `~/.config/affine-flags.conf` (`--ozone-platform=wayland`, `--enable-wayland-ime`).
- **Mã nguồn thực thi:** Quy tắc bảo vệ toàn cục `Safety-First & Reversion Protocol`.