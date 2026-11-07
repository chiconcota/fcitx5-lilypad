# Changelog

All notable changes to **fcitx5-lilypad** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.5.0] - 2026-11-07

### 🌟 Major Release Highlights (Phiên Bản Đột Phá Lớn 2.5)
- **Chuyển dịch Toàn diện sang Bamboo Native Core (Rust C-ABI):** Thay thế hoàn toàn Bamboo Go runtime cũ bằng `bamboo-core` viết bằng Rust nguyên bản, đạt hiệu năng xử lý ký tự cực đại và tiêu thụ bộ nhớ gần như bằng 0.
- **Hardware Sentinel Barrier Protocol (Phím Chốt Chặn Không Phá Hủy KEY_F24):** Thay thế phím chốt chặn Backspace thứ $N+1$ bằng mã quét phần cứng phi phá hủy `KEY_F24`, loại bỏ triệt để nguy cơ xóa lẹm văn bản trước đó khi xảy ra độ trễ hàng đợi Compositor.
- **Dynamic Adaptive Governor & Per-Tab Adaptive Settling:** Cơ chế tự động phân loại ứng dụng và nhịp trễ thông minh:
  - **Terminal / Qt / Native Apps:** Sàn trễ Pre-commit siêu tốc $1.5\text{ms}$ ($1500\mu\text{s}$) và Post-commit $0.3\text{ms}$ ($300\mu\text{s}$) cho cảm giác gõ trực tiếp như bàn phím cơ.
  - **Chromium / Electron / Webview Apps:** Sàn trễ an toàn chuẩn 1 khung hình VSync ($18\text{ms} \sim 22\text{ms}$, tương đương chuẩn $20\text{ms}$ của Lotus) và $5\text{ms}$ Post-commit khi có phím nối tiếp trong RAM. Loại bỏ dứt điểm hiện tượng rụng phím, nuốt chữ âm ghép (`thương`, `thành`) và bung phím thô VNI (`co6 thuo7g`).
- **Virtual Focus Pinning & Focus Bounce Protection:** Tự động bảo vệ bộ đệm từ dở dang khi các ứng dụng web phức tạp (AFFiNE Canvas, Notion, Antigravity IDE, TikTok) phát sinh các đợt giật FocusOut/FocusIn giả lập do cập nhật Virtual DOM.
- **Nhận diện Đa Tầng Process Webview Wayland (`queryCompositorAppId`):** Tích hợp IPC trực tiếp qua `$NIRI_SOCKET` để truy vấn chính xác `app_id` chỉ trong $155\mu\text{s}$, nhận diện hoàn hảo các ứng dụng Wayland chạy trong container / iframe không có metadata.
- **Đồng bộ hóa Trạng thái Ứng dụng trong `setMode()`:** Tự động cập nhật cờ nền tảng Electron và ack uinput ngay khi người dùng chuyển đổi chế độ gõ bằng Menu hoặc Phím tắt.
- **Tái cấu trúc Mã nguồn Chuẩn Modular (< 700 dòng/file):** Phân tách logic trạng thái thành 3 module chuyên trách: `lilypad-state.cpp`, `lilypad-state-governor.cpp`, `lilypad-state-uinput.cpp` theo tiêu chuẩn kỹ thuật nghiêm ngặt.
- **Hỗ trợ Đa Nền tảng Linux:** Chuẩn hóa cấu hình và phụ thuộc để build và chạy mượt mà trên **Arch Linux**, **Ubuntu / Debian**, và **Fedora**.

---

## [2.3.6] - 2026-09-12

### 🚀 Added / Improved (Tính năng Mới & Cải tiến)
- **Đo đạc Thời gian Nuốt phím Thực tế ($\Delta T_{\text{swallow}}$):** Bấm giờ chính xác vòng lặp uinput từ lúc phát chuỗi phím xóa $N+1$ cho tới khi Fcitx5 nuốt phím Sentinel, tự động cập nhật độ trễ nuốt phím theo nhịp thời gian thực qua EMA.
- **Cơ chế Bảo vệ 3 Tầng (Tri-Layer Safety Protection):** Tính toán vi trễ động theo 3 lớp an toàn: $\text{per\_bs\_us} = \max(\text{min\_per\_bs\_us}, \text{ema\_swallow\_us}, T_{\text{measured}})$, vừa tôn trọng tốc độ gõ của người dùng vừa cấp cứu ngay lập tức khi ứng dụng bị nghẽn (spike/lag).
- **Chuẩn hóa Watchdog Hard Timeout 250ms Toàn Hệ thống:** Áp dụng trần bảo vệ 250ms trên Linux EventLoop, đảm bảo bàn phím không bao giờ bị đơ/kẹt khi ứng dụng đích bị treo.
- **Post-Commit Settling Window cho Chromium/Electron:** Duy trì trạng thái giữ phím an toàn trong RAM ($70\text{ms}$) sau khi commit trên các ứng dụng Webview, giải quyết triệt để lỗi nuốt chữ âm ghép ("thương" $\to$ "tương").

### 🧹 Refactored / Clean Code (Dọn dẹp Nợ Kỹ thuật)
- **Loại bỏ Hoàn toàn Cờ Riêng `is_antigravity_flag_`:** Xóa bỏ toàn bộ các mức sàn cứng ad-hoc $32\text{ms}$ và nhánh rẽ đặc thù, giữ cho core engine Fcitx5 trong sạch và thuần khiết.
- **Thống nhất Cơ chế Nhận diện Chromium (`ack_apps`):** Đưa Antigravity và các ứng dụng nền tảng Electron về cơ chế Chromium tiêu chuẩn (`wa_chromium_flag = true`), trao quyền điều phối hoàn toàn cho kiến trúc Adaptive Dynamic Micro-Pacing tự thích ứng.

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
