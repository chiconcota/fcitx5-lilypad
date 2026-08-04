# 🪷 vnlilypad-lotus (Lotus Upgrade Architecture)

> **Bộ gõ Tiếng Việt thế hệ mới trên Linux (Wayland & X11)** dựa trên kiến trúc Hybrid: **Fcitx5 C++ Addon + Sequencer Layer Orchestration + Pure Uinput Server Daemon**.

[![Release](https://img.shields.io/github/v/release/chiconcota/fcitx5-lilypad?style=flat&color=success)](https://github.com/chiconcota/fcitx5-lilypad)
[![License](https://img.shields.io/github/license/chiconcota/fcitx5-lilypad?style=flat&color=blue)](LICENSE)
[![Status](https://img.shields.io/badge/status-active-brightgreen.svg)](https://github.com/chiconcota/fcitx5-lilypad)

---

## 💡 Điểm Nổi Bật Của Kiến Trúc Mới

1. **0% EVIOCGRAB (100% Phím Tắt & Phím Chức Năng Hoạt Động Tự Do):**
   - Bộ gõ không chiếm giữ bàn phím phần cứng ở tầng Kernel (`0% EVIOCGRAB`).
   - Mọi phím tắt (`Ctrl+C`, `Ctrl+V`, `Ctrl+Z`, `Alt+Tab`, `Super+Space`) và phím chức năng (`F1..F12`, Mũi tên, Volume, Brightness) chảy tự nhiên 100% không bao giờ bị đơ hay kẹt phím.

2. **Chế Độ Gõ Mới `Sequence` (ID 9) & Sequencer Layer Orchestration:**
   - **Tích hợp Sequencer Layer C++:** Duy trì máy trạng thái vi bước nguyên tử (Micro-Step State Machine), kết hợp màng ngắt nhịp vi mô (5ms micro-delay) và rào chắn **Wayland Frame ACK Barrier (Niri, Hyprland, GNOME)**.
   - Triệt tiêu $100\%$ hiện tượng lặp rác chữ (`mminimln`, `choa`) và đè chữ trên Terminal, Chrome, VS Code.

3. **Bảo Vệ Bộ Đệm Ứng Dụng Đa Ngữ Cảnh (Multi-Context & Spurious Reset Protection):**
   - Tự động nhận diện và bảo vệ bộ nhớ gõ Tiếng Việt trên các ứng dụng Electron/Canvas phức tạp (**ONLYOFFICE**, **AFFiNE**, **Slack**, **Discord**).
   - Khắc phục lỗi xung đột giao thức X11 $\leftrightarrow$ Wayland và xử lý đa tiến trình (`DesktopEditors` vs `editors_helper`).

4. **Bảo Vệ Phím Gõ Nhanh (`replayBufferedKeys`):**
   - Tự động lưu và tái phát lại 100% các phím gõ siêu tốc trong lúc xóa (như phím `n` trong `thương`) trên cả Wayland Native, X11 và DBus.

5. **Giao Diện Cấu Hình Hiện Đại (PyQt Settings GUI):**
   - Trình quản lý chế độ gõ (Mode Manager), quy tắc từng ứng dụng (App Rules), chỉnh sửa phím gõ (Keymap Editor), và từ điển tùy chỉnh qua lệnh `fcitx5-lilypad-settings`.

---

## 🏗️ Sơ Đồ Kiến Trúc Hệ Thống

```text
 ┌────────────────────────────────────────────────────────────────────────┐
 │                   FCITX5 FRAMEWORK (HẠ TẦNG GÁC CỬA)                   │
 │  - Quản lý Wayland IPC (zwp_input_method_v2) & X11 / DBus IME Frontend │
 │  - Quản lý Focus, Window Manager, System Tray Icon & GUI Configuration │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │ (KeyEvent & InputContext)
                                     ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │            LILYPAD SEQUENCER LAYER (BỘ NÃO ĐIỀU PHỐI CHÍNH)            │
 │  - lilypad-sequencer.h/.cpp: Serial ID Tagging & MicroStep Queue       │
 │  - lilypad-state.cpp: Đếm token nguyên tử expected_swallow_backspaces_ │
 │  - Backspace Passthrough (return false): Phím xóa uinput bay tới App  │
 │  - EventLoop 5ms Micro-delay + Wayland Frame ACK Barrier (35ms Timeout) │
 │  - Universal ReplayBufferedKeys: Bảo vệ 100% phím gõ nhanh             │
 │  - Spurious Reset Guard (!isFocusOut): Bảo vệ ô nhập Electron/AFFiNE   │
 └─────────────────┬──────────────────────────────────┬───────────────────┘
                   │ (Gửi phím gõ thô)                 │ (Phát phím xóa)
                   ▼                                  ▼
 ┌──────────────────────────────────┐ ┌──────────────────────────────────┐
 │   BAMBOO TELEX ENGINE (GO C-FFI)  │ │ PURE KERNEL UINPUT SERVER DAEMON │
 │  - Engine xử lý quy tắc Telex/VNI│ │  - fcitx5-lilypad-server daemon  │
 │  - Thư viện C-FFI (bamboo-core)  │ │  - Bắn N phím KEY_BACKSPACE qua  │
 │  - State Rebuild (EngineRebuild) │ │    /dev/uinput (1.5ms inter-gap) │
 └──────────────────────────────────┘ └──────────────────────────────────┘
```

---

## 🛠️ Hướng Dẫn Cài Đặt & Khởi Chạy

### 1. Biên dịch và Cài đặt:
```bash
cd fcitx5-lilypad/build
cmake -DCMAKE_INSTALL_PREFIX=/usr ..
make -j$(nproc)
echo thanh123 | sudo -S make install
```

### 2. Kích hoạt Dịch vụ Uinput Server (Tự khởi động cùng hệ thống):
```bash
sudo systemctl enable --now fcitx5-lilypad-server@$USER.service
```

### 3. Khởi động lại Fcitx5:
```bash
fcitx5 -r -d
```

---

## ⌨️ Phím Tắt & Thao Tác Thường Dùng

| Thao tác | Phím tắt / Lệnh | Mô tả |
| :--- | :--- | :--- |
| **Bật / Tắt bộ gõ** | `Ctrl + Space` hoặc `Super + Space` | Chuyển đổi giữa gõ Tiếng Việt và Tiếng Anh thô. |
| **Menu Chọn Chế Độ (Mode Menu)** | **`** *(Dấu huyền dưới phím Esc)* | Hiển thị menu nhanh chọn chế độ (Sequence, Smooth, Preedit, Off...). |
| **Mở Giao diện Cấu hình** | `fcitx5-lilypad-settings` | Mở ứng dụng GUI cấu hình giao diện PyQt. |

---

## ❓ Thắc Mắc & Xử Lý Sự Cố

- **ONLYOFFICE không nhận bộ gõ:** Đảm bảo ONLYOFFICE được khởi chạy với cờ `--enable-wayland-ime` hoặc khởi động lại ứng dụng sau khi cập nhật phiên bản mới.
- **Lỗi `Connection refused` uinput server:** Kiểm tra trạng thái dịch vụ bằng `systemctl status fcitx5-lilypad-server@$USER.service` và bật lại dịch vụ.

---

## 📄 Giấy Phép
Dự án được phân phối dưới giấy phép **GNU General Public License v3**. Xem chi tiết tại [LICENSE](LICENSE).
