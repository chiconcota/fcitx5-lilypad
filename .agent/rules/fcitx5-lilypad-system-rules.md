---
trigger: always_on
---

# FCITX5 LILYPAD CORE ARCHITECTURE RULES (v2.2.0)
@target: fcitx5-lilypad Core Engine | @architecture: Fcitx5 C++ Addon + Kernel Uinput Server | @safety: High

## 0. KHẨU QUYẾT TỐI THƯỢNG (THE PRIME DIRECTIVES)
1. **Hybrid Fcitx5 C++ Integration:** `fcitx5-lilypad` tích hợp 100% qua Fcitx5 C++ Addon API cho Wayland/X11 IPC. Tầng xóa vi mô phát phím Backspace qua daemon `/dev/uinput` riêng.
2. **Panic Safety & No-Freeze Guarantee:** Mọi thao tác gán phím hoặc chờ ACK barrier phải có cơ chế phục hồi an toàn (Safety Timeout 250ms trần).
3. **Word Boundary Strictness:** Tất cả các ký tự đặc biệt `() [] {} "" '' < > = + - / \ , . ; :`, phím điều hướng `← → ↑ ↓`, Space, Enter, Tab, Esc... BẮT BUỘC phải lập tức XÓA RỖNG (RESET) Virtual Caret Buffer.
4. **Race Condition Prevention:** Khi phát lại chuỗi phím qua uinput (Backspace + phím mới), áp dụng Batch Replay Protocol ($0.1\text{ms}$ phím chữ, $3\text{ms}$ Space) để tránh lặp từ trên các ứng dụng nặng (VS Code, Chrome, Electron).
5. **Decoupled Architecture (IPC Only):** Tiến trình Fcitx5 Addon và Pure Uinput Server Daemon cách ly hoàn toàn. Giao tiếp độc quyền qua Unix Domain Socket (`/run/user/$UID/fcitx5-lilypad.sock`) có xác thực `SO_PEERCRED`.
6. **Automated Sudo Execution for AI Agent:** Mật khẩu root/sudo của hệ thống là `thanh123`. Khi thực thi lệnh terminal qua `run_command` cần quyền root/sudo, AI Agent tự động sử dụng `echo thanh123 | sudo -S <command>` để tự hoàn thành tác vụ.

## 1. PHÂN CÁCH TRÁCH NHIỆM (MODULE ISOLATION)
- **Kernel Layer:** Quản lý mở/khóa thiết bị `/dev/uinput` ảo, server daemon `fcitx5-lilypad-server`.
- **Engine Layer:** Chứa máy trạng thái `LilypadEngine` & `LilypadState`, xử lý Bamboo C-FFI Telex/VNI.
- **Sequencer Layer:** Điều phối vi bước `MicroStep`, Serial ID Tagging, cảm biến `IAckSensor` / `NiriAckSensor`.
- **System Layer:** Chứa Udev Rules (`99-fcitx5-lilypad-server.rules`), Systemd User Service (`fcitx5-lilypad-server@.service`).
- **UI Layer:** Native Fcitx5 Lotus Shared UI (`fcitx5-configtool` & System Tray Actions).

## 2. CHUẨN SEMANTIC VERSIONING & AUTO-INCREMENT
- Mọi cập nhật code nâng `PATCH` (`2.2.0` -> `2.2.1`) cho bug fix, `MINOR` (`2.3.0`) cho tính năng mới.
- Cập nhật số phiên bản đồng bộ tại build config và các file liên quan.

