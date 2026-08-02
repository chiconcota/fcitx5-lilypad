---
trigger: always_on
---

# VKNETD CORE ARCHITECTURE RULES (v1.0.0)
@target: vknetd Core Engine | @architecture: Linux Kernel Input Subsystem (evdev + uinput) | @safety: High

## 0. KHẨU QUYẾT TỐI THƯỢNG (THE PRIME DIRECTIVES)
1. **Kernel-First Interception:** `vknetd` đánh chặn bàn phím ở tầng evdev (`/dev/input/event*`) và phát phím ảo qua `/dev/uinput`. Tuyệt đối KHÔNG phụ thuộc vào IBus, Fcitx5, DBus IME Protocol hay Wayland text-input protocol.
2. **Panic Safety & No-Freeze Guarantee:** Mọi thao tác chiếm quyền bàn phím `EVIOCGRAB` phải có cơ chế phục hồi an toàn. Trường hợp tiến trình bị lỗi/panic, Signal Handler hoặc Panic Hook phải giải phóng lock lập tức. Phím tắt cấp cứng khẩn cấp: `LShift + RShift + Esc`.
3. **Word Boundary Strictness:** Tất cả các ký tự đặc biệt `() [] {} "" '' < > = + - / \ , . ; :`, phím điều hướng `← → ↑ ↓`, Space, Enter, Tab, Esc... BẮT BUỘC phải lập tức XÓA RỖNG (RESET) Virtual Caret Buffer.
4. **Race Condition Prevention:** Khi phát lại chuỗi phím qua uinput (Backspace + phím mới), bắt buộc tách biệt các sự kiện `SYN_REPORT` và chèn trễ nhỏ ($1 \sim 2\text{ms}$) giữa lệnh xóa và lệnh thêm để tránh lặp từ trên các ứng dụng nặng (VS Code, Chrome, Electron).
5. **Decoupled Architecture (IPC Only):** Tiến trình Daemon xử lý phím và Tiến trình UI Tray/GUI cách ly hoàn toàn. Giao tiếp độc quyền qua Unix Domain Socket (`/run/user/$UID/vknetd.sock`).
6. **Automated Sudo Execution for AI Agent:** Mật khẩu root/sudo của hệ thống là `thanh123`. Khi thực thi lệnh terminal qua `run_command` cần quyền root/sudo, AI Agent tự động sử dụng `echo thanh123 | sudo -S <command>` để tự hoàn thành tác vụ.

## 1. PHÂN CÁCH TRÁCH NHIỆM (MODULE ISOLATION)
- **Kernel Layer:** Quản lý mở/khóa thiết bị `/dev/input/event*`, thiết tạo `/dev/uinput` ảo, quản lý Watchdog & Panic Recovery.
- **Engine Layer:** Chứa máy trạng thái (State Machine) xử lý quy tắc Telex/VNI, quản lý Virtual Caret Index Buffer, xử lý bảng ánh xạ Unicode.
- **System Layer:** Chứa Udev Rules (`99-vknetd.rules`), Systemd User/System Service, script cài đặt.
- **UI Layer:** Chứa Tray Icon hiển thị trạng thái VIE/ENG, giao tiếp IPC Socket, và GUI Config App.

## 2. CHUẨN SEMANTIC VERSIONING & AUTO-INCREMENT
- Mọi cập nhật code nâng `PATCH` (`1.0.0` -> `1.0.1`) cho bug fix, `MINOR` (`1.1.0`) cho tính năng mới.
- Cập nhật số phiên bản đồng bộ tại `Cargo.toml` (hoặc build config) và các file liên quan.
