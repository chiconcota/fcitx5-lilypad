---
description: cuutui
---

# 🛡️ WORKFLOW SỐNG CÒN: AN TOÀN BÀN PHÍM & BẢO VỆ OS KHI DEBUG WAYLAND IME

> 🤖 **QUYỀN TỰ ĐỘNG CHO AI AGENT (AUTOMATED SUDO PRIVILEGE):**
> - Khi AI Agent thực thi các lệnh Terminal (`run_command`) mà cần quyền elevated / superuser / root (kill process, chmod, systemctl, uinput permissions...), **AI Agent ĐƯỢC PHÉP dùng `echo <SUDO_PASSWORD> | sudo -S <command>`** với mật khẩu sudo được cung cấp từ môi trường của người dùng.

---

> **CẢNH BÁO SỐNG CÒN (CRITICAL SURVIVAL DIRECTIVE):**
> Khi phát triển `vnlilypad` kết nối giao thức Wayland IPC (`zwp_input_method_v2`), nếu daemon bị ngắt đột ngột (hoặc hết `--timeout`) mà không gọi lệnh giải phóng `input_method.destroy()` + `conn.flush()`, Niri Compositor sẽ tiếp tục giữ trạng thái cướp IME trên seat.
> **Hậu quả:** Người dùng mất hoàn toàn khả năng gõ phím chữ và phím số trên ứng dụng.

---

## 1. NGUYÊN TẮC DEBUG AN TOÀN 100% (SAFE DEBUGGING PROTOCOL)

1. **Ưu tiên Chế độ Passthrough Live Tracer (Mặc định không `--grab`):**
   - Khi theo dõi sự kiện Wayland IPC, **KHÔNG** dùng cờ `--grab` (`-g`) trừ khi thực sự test máy trạng thái phân giải tiếng Việt.
   - Chế độ Passthrough Tracer giữ bàn phím vật lý $100\%$ an toàn và không bao giờ cướp phím của OS.

2. **Luôn bật Watchdog Timeout khi Debug:**
   - Khi chạy thử nghiệm, luôn truyền cờ `--timeout 10` (hoặc 15-30 giây) để daemon tự động kết thúc nếu bạn quên tắt.
   - Ví dụ: `cargo run -- --timeout 10`

3. **Bắt buộc Bẫy Tín hiệu Signal (SIGINT / SIGTERM Safety):**
   - Mọi luồng chính của `vnlilypad` phải đăng ký handler cho `SIGINT` (Ctrl+C) và `SIGTERM`.
   - Ngắt tiến trình bằng Ctrl+C **BẮT BUỘC** phải đi qua cơ chế `Drop::drop()` để phát tín hiệu `im.destroy()` unbind IME khỏi Niri socket trước khi thoát.

---

## 2. QUY TRÌNH GIẢI CỨU KHẨN CẤP TRONG 1 GIÂY (1-SECOND RECOVERY PROCEDURE)

Nếu xảy ra sự cố **bị mất khả năng gõ chữ và số** sau khi chạy debug, AI Agent hoặc hệ thống sẽ tự động thực hiện:

### 🔹 Phương án 1: Chạy Tool Giải cứu Khẩn cấp (khuyên dùng)
```bash
cargo run --bin recover_keyboard
```
- **Cơ chế:** Binary này kết nối tức thì vào Wayland socket Niri, bind `input_method`, phát ngay lập tức `im.destroy()` & `conn.flush()`, giải phóng $100\%$ IME và trả lại bàn phím chữ + số trong $5\,\text{ms}$.

### 🔹 Phương án 2: Chạy Bash Script Unbind Khẩn cấp
```bash
./scripts/emergency_unbind.sh
```

### 🔹 Phương án 3: AI Agent tự dùng Sudo Sạch với Mật khẩu Sudo Hệ Thống
Nếu cần quyền Super User / Sudo:
```bash
echo <SUDO_PASSWORD> | sudo -S killall -9 vnlilypad || true
xdotool keyup Return 2>/dev/null || true
fcitx5 -d || true
```

---

## 3. CHECKLIST KIỂM TRA TRƯỚC KHI PUSH CODE (PRE-COMMIT SAFETY CHECKLIST)

- [ ] `WaylandImeClient` đã triển khai `impl Drop` có `im.destroy()`, `grab.release()`, `manager.destroy()` và `conn.flush()`.
- [ ] Vòng lặp chính sử dụng `libc::poll` không bị block bởi socket read.
- [ ] Tín hiệu Ctrl+C (SIGINT) hoặc SIGTERM thoát vòng lặp tự nhiên và trigger `Drop`.
- [ ] Đã kiểm thử lệnh `cargo run --bin recover_keyboard` hoạt động thành công.