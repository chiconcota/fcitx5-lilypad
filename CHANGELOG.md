# Changelog

All notable changes to **fcitx5-lilypad** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [2.3.1] - 2026-08-28

### ⚡ Changed / Improved (Cải tiến & Tối ưu)
- **Chuẩn hóa Hướng dẫn Cài đặt AUR:** Bổ sung hướng dẫn 2 bước chi tiết (Cài đặt gói & Kích hoạt Daemon qua Systemd) trong cả 3 tài liệu `README.md`, `fcitx5-lilypad/README.md`, và `fcitx5-lilypad/README.en.md`.
- **Cải tiến Scriptlet `fcitx5-lilypad.install`:** Tự động đóng khung hướng dẫn bật `fcitx5-lilypad-server@$USER.service` và khởi động lại Fcitx5 (`fcitx5 -r -d`) ngay trên terminal khi cài đặt qua `pacman`/`yay`.
- **Đồng bộ hóa Gói Đóng Gói AUR:** Cập nhật phiên bản và mã băm SHA256 cho 3 gói AUR (`fcitx5-lilypad`, `fcitx5-lilypad-bin`, `fcitx5-lilypad-git`).

---

## [2.3.0] - 2026-08-26

### 🚀 Added (Mới)
- **Cảm biến nhịp gõ IKI (`IIkiSensor` & `StandardIkiSensor`):** Đo liên tục nhịp gõ ngón tay thời gian thực qua thuật toán làm mịn EMA.
- **Giao thức Sentinel Barrier $N+1$:** Phát $N+1$ phím xóa qua `/dev/uinput` và nuốt trọn phím thứ $N+1$ làm chốt chặn an toàn FIFO trước khi commit.
- **Cơ chế Two-Tier Timeout:** Dynamic Soft Timeout (nhận diện app lag để gom phím vào RAM) kết hợp Watchdog Hard Timeout 250ms và Cắt lỗ khẩn cấp (`purgeContextEmergency`).
- **Cold Start Safe Baseline:** Thiết lập ngưỡng an toàn $>50\text{ms}$ ($50\text{ms} \sim 80\text{ms}$) cho ký tự đầu tiên khi chưa có lịch sử dữ liệu IKI và App ACK, bảo đảm 100% không nuốt chữ ở từ đầu.
- **Hạ tầng đóng gói AUR:** Bổ sung PKGBUILD chuẩn hóa cho `fcitx5-lilypad`, `fcitx5-lilypad-bin`, và `fcitx5-lilypad-git`.

### ⚡ Changed / Improved (Cải tiến & Tối ưu)
- **Dynamic Micro-Pacing via Normalized Lerp:** Thay thế vi trễ cố định bằng thuật toán nội suy tuyến tính kết hợp thời gian tiêu thụ DOM của App ($N \times T_{\text{ack}}$) — nén xuống $1.5\text{ms} \sim 2.5\text{ms}$ trên Terminal (Zero-Latency) và mở rộng $45\text{ms} \sim 60\text{ms}$ trên Web DOM.
- **Uniform Web IME Routing:** Đồng bộ hóa kênh phát qua `ic_->commitString()` cho Chromium/Electron chống xung đột Virtual DOM.
- **Độ chính xác Timer $1\mu\text{s}$:** Đặt độ chính xác $1\mu\text{s}$ cho Event Loop timer trên Linux Kernel.
- **Nâng phiên bản `2.3.0`:** Đồng bộ CMakeLists.txt, about.py và tài liệu.

### 🐛 Fixed (Sửa lỗi)
- **Khắc phục lỗi nuốt chữ trên Facebook / Web DOM:** Triệt tiêu 100% race condition phím xóa Backspace đè lên ký tự mới commit.
- **Khắc phục lỗi đảo dấu cách trên GTK4 / Gnome Text Editor:** Giữ nguyên kênh Native key với độ chính xác timer $1\mu\text{s}$.
- **Khắc phục lỗi kẹt/đơ bàn phím khi App bị treo:** Tự động reset và xả phím thô an toàn sau 250ms qua Watchdog timer.

---

## [2.2.0] - 2026-08-09

### 🚀 Added (Mới)
- **Kiến trúc Modular IAckSensor:** Tách rời lớp cảm biến thành các module cắm/rút (`IAckSensor`, `NiriAckSensor`, `GenericAckSensor`, `AckSensorFactory`) tự động nhận diện theo môi trường desktop.
- **Bộ nhận diện thương hiệu Lá Súng (Lilypad Leaf):** Thiết kế bộ icon vector SVG mới và làm mới giao diện trang About Qt.

### ⚡ Changed / Improved (Cải tiến & Tối ưu)
- **Nhúng trực tiếp 100% Bamboo-Core Go Source:** Tích hợp 19 file mã nguồn Go vào repository, loại bỏ hoàn toàn lỗi submodule rỗng khi `git clone`.
- **Tối ưu nhịp Replay 3000:300:** Đặt vi trễ $0.3\text{ms}$ cho phím chữ và $3\text{ms}$ cho phím Space, triệt tiêu hiện tượng đè rác chữ khi gõ siêu tốc.
- **Release Zero-Log Overhead:** Tự động vô hiệu hóa log debug khi build bản Release (`NDEBUG`).

### 🐛 Fixed (Sửa lỗi)
- **Sửa cấu hình Addon Config:** Sửa `Library=liblilypad` và định dạng `[Dependencies]` trong file `.conf.in.in`.
- **Sửa đường dẫn Systemd Service:** Cập nhật đường dẫn cài đặt service unit về `/lib/systemd/system`.

---

## [2.1.0] - 2026-08-04

### 🚀 Added (Mới)
- **Kernel Uinput Server Daemon (`fcitx5-lilypad-server`):** Daemon độc lập phát phím Backspace nguyên tử qua `/dev/uinput` với xác thực UID Unix Domain Socket (`SO_PEERCRED`).
- **Chế độ gõ Sequence (Enum ID 9):** Tích hợp Sequencer Layer C++, Serial ID Tagging và bộ đếm token xóa nguyên tử.
- **Systemd User Service:** Hỗ trợ kích hoạt per-user qua `fcitx5-lilypad-server@.service`.

### 🐛 Fixed (Sửa lỗi)
- **Loại bỏ `deleteSurroundingText` & Preedit:** Giải quyết triệt để lỗi nhảy con trỏ và xung đột văn bản trên Wayland.
