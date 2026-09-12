# 🍃 fcitx5-lilypad

> **Bộ gõ Tiếng Việt thế hệ mới cho Linux Wayland & X11** dựa trên kiến trúc Hybrid: **Fcitx5 C++ Addon + IKI Adaptive Engine + Modular IAckSensor + Sentinel Barrier Protocol + Pure Kernel Uinput Server Daemon**.

[![Release](https://img.shields.io/github/v/release/chiconcota/fcitx5-lilypad?style=flat&color=success)](https://github.com/chiconcota/fcitx5-lilypad/releases)
[![Version](https://img.shields.io/badge/version-2.3.6-blue.svg)](https://github.com/chiconcota/fcitx5-lilypad)
[![License](https://img.shields.io/github/license/chiconcota/fcitx5-lilypad?style=flat&color=blue)](LICENSE)
[![Tested Compositor](https://img.shields.io/badge/tested_compositor-Niri-purple.svg)](https://github.com/niri-wm/niri)
[![Target Packaging](https://img.shields.io/badge/target_packaging-AUR_Arch_Linux-blue.svg)](#-phát-hành--cài-đặt)
[![Community Status](https://img.shields.io/badge/community-call_for_testers-orange.svg)](#-kêu-gọi-cộng-đồng-đóng-góp--thử-nghiệm-call-for-testers)

---

## 💡 Điểm Đột Phá Kiến Trúc (`v2.3.6 - Adaptive Dynamic Micro-Pacing & Tri-Layer Protection`)

`fcitx5-lilypad` v2.3.6 giải quyết dứt điểm các vấn đề cố hữu của bộ gõ tiếng Việt trên Linux (nuốt chữ, mất ký tự khi gõ nhanh, đè rác chữ trên Web DOM/Electron, và đảo dấu cách) thông qua các công nghệ cốt lõi:

```text
  ┌────────────────────────────────────────────────────────────────────────┐
  │                   FCITX5 FRAMEWORK (HẠ TẦNG GÁC CỬA)                   │
  │  - Quản lý Wayland IPC (zwp_input_method_v2) & X11 / DBus IME Frontend │
  │  - Đo nhịp tay người dùng thời gian thực (IIkiSensor EMA Tracking)     │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │ (KeyEvent & IKI Speed)
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │         LILYPAD SEQUENCER & SENTINEL BARRIER (BỘ NÃO ĐIỀU PHỐI)        │
  │  - Sentinel Barrier N+1: Bắn N+1 Backspace, nuốt phím thứ N+1 bảo vệ   │
  │  - Đo đạc thời gian nuốt thực tế: ΔT_swallow = T2 - T1 (nguồn -> đích) │
  │  - Post-Commit Settling Window (70ms): Giữ phím RAM cho Chromium Web   │
  │  - Watchdog Hard Timeout (250ms) & Emergency Purge: Chống đơ phím 100% │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │          MODULAR ACK SENSOR LAYER (CẢM BIẾN THÍCH ỨNG ĐỘ TRỄ)          │
  │  - Tri-Layer Protection: max(Sàn ngón tay, EMA swallow, T_measured)   │
  │  - Dynamic Micro-Pacing: T_tổng = base_us + N * per_bs_us              │
  │  - Cold Start Safe Baseline: Trần trễ an toàn cho chữ đầu tiên         │
  └───────────────────────────────────┬────────────────────────────────────┘
                    │ (Gửi phím gõ thô)                 │ (Phát N+1 phím xóa)
                    ▼                                  ▼
  ┌──────────────────────────────────┐ ┌──────────────────────────────────┐
  │   BAMBOO TELEX ENGINE (GO C-FFI)  │ │ PURE KERNEL UINPUT SERVER DAEMON │
  │  - Engine xử lý quy tắc Telex/VNI│ │  - fcitx5-lilypad-server daemon  │
  │  - Thư viện C-FFI (bamboo-core)  │ │  - Bắn mảng ev[4] KEY_BACKSPACE  │
  │  - State Rebuild tức thì         │ │    qua /dev/uinput nguyên tử     │
  └──────────────────────────────────┘ └──────────────────────────────────┘
```

### 1. Đo Đạc Nuốt Phím Thực Tế ($\Delta T_{\text{swallow}}$) & Cơ Chế Bảo Vệ 3 Tầng (Tri-Layer Protection)
- **Bấm giờ vòng lặp uinput thực tế:** Hệ thống bấm giờ từ lúc phát chuỗi phím xóa $N+1$ cho tới khi phím Sentinel quay về Fcitx5 để tính $\Delta T_{\text{swallow}}$ và độ trễ thô mỗi phím $T_{\text{measured}} = \frac{\Delta T_{\text{swallow}}}{N+1}$.
- **Cơ chế bảo vệ 3 tầng (Tri-Layer Safety Protection):**
  $$\text{per\_bs\_us} = \max\Big(\underbrace{\text{min\_per\_bs\_us}}_{\text{Lớp 1: Sàn ngón tay } t}, \quad \underbrace{\text{app\_ack}_{\text{new}}}_{\text{Lớp 2: EMA mượt mà}}, \quad \underbrace{T_{\text{measured}}}_{\text{Lớp 3: Phản xạ tức thì}}\Big)$$
  - **Lớp 1 (Sàn ngón tay):** Tự động co giãn theo nhịp gõ $\text{IKI} \in [35\text{ms}, 150\text{ms}]$. Khi gõ lướt cực nhanh (Burst), nén vi trễ xuống **$1.0\text{ms} \sim 2.5\text{ms}$** (Zero-Latency tức thì trên Terminal).
  - **Lớp 2 (EMA làm mịn):** Giữ nhịp gõ ổn định, triệt tiêu xung nhiễu giật cục của hệ thống.
  - **Lớp 3 (Phản xạ tức thì):** Tự động dãn vi trễ trong $1\mu\text{s}$ để cấp cứu ký tự nếu ứng dụng đột ngột bị giật/lag ở chính lần gõ đó.

### 2. Giao Thức Uinput Sentinel Barrier $N+1$ & Post-Commit Settling Window
- Khi thay thế ký tự, daemon phát **$N+1$ phím xóa `KEY_BACKSPACE`**:
  - $N$ phím đầu xóa ký tự cũ trong ứng dụng.
  - Phím thứ $N+1$ (Sentinel) được Fcitx5 nuốt trọn làm chốt chặn an toàn vật lý FIFO trước khi commit.
- **Post-Commit Settling Window (70ms):** Khi gõ trên các trình duyệt và ứng dụng Webview (Chromium, Brave, Edge, VS Code), Fcitx5 giữ phím gõ nhanh tiếp theo trong RAM trong $70\text{ms}$ để cây DOM hoàn tất render ký tự vừa commit, loại bỏ $100\%$ lỗi nuốt chữ âm ghép (như `"thương"` $\to$ `"tương"`).

### 3. Watchdog Hard Timeout (250ms) Chuẩn Hóa & Clean Architecture
- **Watchdog Hard Timeout (250ms):** Chuẩn hóa mức trần an toàn $250\text{ms}$ độc lập trên Linux EventLoop. Nếu ứng dụng bị đóng băng (freeze), hệ thống lập tức kích hoạt `purgeContextEmergency()` xả toàn bộ phím đệm ra màn hình, bảo đảm **bàn phím không bao giờ bị đơ hay kẹt cứng**.
- **Clean Code & Zero Technical Debt:** Loại bỏ hoàn toàn các cờ ngoại lệ cứng, đưa toàn bộ ứng dụng nền tảng Chromium/Electron về cơ chế chuẩn hóa qua danh mục `ack_apps`.

---

## 💖 Lời Cảm Ơn (Acknowledgments)

Dự án **fcitx5-lilypad** xin gửi lời tri ân sâu sắc đến những đóng góp quý giá đã đặt nền móng cho sự phát triển của bộ gõ:

* **Tác giả Engine Bamboo:** Chân thành cảm ơn tác giả **Luật Nguyễn** ([BambooEngine](https://github.com/BambooEngine/bamboo-core)) đã phát triển bộ engine Bamboo mã nguồn mở tuyệt vời — trái tim thuật toán xử lý biến âm Tiếng Việt tự nhiên và chuẩn xác.
* **Tác giả bộ gõ `fcitx5-lilypad`:** Tác giả **Võ Ngô Hoàng Thành** ([thanhpy2009 / VMK](https://github.com/thanhpy2009)) — Kiến trúc sư trưởng thiết kế hạ tầng Sequencer, Sentinel Barrier $N+1$, Uinput Server Daemon, Cảm biến IKI Adaptive và cơ chế điều hòa vi trễ Tri-Layer Protection.
* **Dự án tiền đề `fcitx5-lotus`:** Chân thành cảm ơn dự án [fcitx5-lotus](https://github.com/vnlilypad/fcitx5-lotus) — Nguồn cảm hứng mở đường và nền móng vững chắc ban đầu cho hành trình xây dựng bộ gõ tiếng Việt hiện đại, mượt mà trên Linux Wayland & X11.

---

## 🎯 Trạng Thái Hiện Tại & Môi Trường Thử Nghiệm

Bộ gõ được tối ưu hóa chuyên sâu trên **Niri Compositor (Arch Linux)** và hỗ trợ vạn năng (Universal) trên tất cả các Compositor Wayland & X11 thông qua `GenericAckSensor`.

### 📊 Ma Trận Tương Thích (Compositor & Distro Matrix)

| Window Compositor | Trạng thái | Ghi chú | Nhu cầu đóng góp |
| :--- | :---: | :--- | :---: |
| **Niri** | 🟢 **Sẵn sàng** | Tích hợp `NiriAckSensor` + IKI Adaptive + Sentinel Barrier | Kiểm thử liên tục |
| **Hyprland** | 🟢 **Sẵn sàng** | Chạy vạn năng qua `GenericAckSensor` | Tester & Feedback |
| **Sway** | 🟢 **Sẵn sàng** | Hỗ trợ qua Wayland `zwp_input_method_v1/v2` | Tester & Feedback |
| **KDE Plasma (Wayland)**| 🟢 **Sẵn sàng** | Chạy qua KWin Wayland IME API | Tester & Feedback |
| **GNOME (Wayland)** | 🟢 **Sẵn sàng** | Chạy qua Mutter Wayland Input API | Tester & Feedback |
| **X11 (Generic)** | 🟢 **Sẵn sàng** | Fallback qua Fcitx5 X11 Frontend + `GenericAckSensor` | Tester & Feedback |

---

## 📦 Phát Hành & Cài Đặt (Installation Guide)

### 1. Cài đặt từ AUR (Arch Linux / Manjaro / EndeavourOS)

Người dùng Arch Linux có thể cài đặt dễ dàng qua các helper AUR (`yay` hoặc `paru`):

**Bước 1: Cài đặt gói bộ gõ:**
```bash
# 1. Bản Binary phát hành chính thức - Cài tức thì 1s, TẮT LOG 100% (Khuyên dùng cho người dùng cuối):
yay -S fcitx5-lilypad-bin

# 2. Bản Source phát hành chính thức - Tự biên dịch từ Release Tarball, TẮT LOG:
yay -S fcitx5-lilypad

# 3. Bản Git mới nhất - Tự động cập nhật theo commit nhánh main, BẬT FULL LOG (Dành cho Dev & Tester):
yay -S fcitx5-lilypad-git
```

**Bước 2: Kích hoạt Uinput Daemon Service & Khởi động lại Fcitx5:**
```bash
# Kích hoạt daemon chạy ngầm theo user (chỉ cần thực hiện 1 lần duy nhất):
sudo systemctl enable --now fcitx5-lilypad-server@$USER.service

# Khởi động lại Fcitx5 để nhận diện bộ gõ:
fcitx5 -r -d
```

### 2. Biên dịch và Cài đặt từ Mã Nguồn (Build from Source)

#### Bước 0: Cài đặt các gói phụ thuộc (Dependencies)

- **Ubuntu / Debian:**
  ```bash
  sudo apt update && sudo apt install -y git fcitx5 fcitx5-config-qt libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev fcitx5-modules-dev libinput-dev libudev-dev extra-cmake-modules build-essential cmake gettext golang libx11-dev python3 python3-qtpy python3-pyqt5
  ```

- **Arch Linux:**
  ```bash
  sudo pacman -S --needed git base-devel cmake extra-cmake-modules fcitx5 fcitx5-configtool fcitx5-qt fcitx5-gtk gettext fmt go libx11 libinput systemd python python-qtpy python-pyqt5
  ```

- **Fedora:**
  ```bash
  sudo dnf install -y git gcc-c++ cmake extra-cmake-modules fcitx5 fcitx5-devel fcitx5-configtool fcitx5-autostart fcitx5-gtk fcitx5-qt5 fcitx5-qt6 gettext-devel golang libX11-devel libinput-devel systemd-devel python3 python3-QtPy python3-qt5
  ```

#### Bước 1: Clone và Biên dịch

```bash
# 1. Clone repository
git clone https://github.com/chiconcota/fcitx5-lilypad.git
cd fcitx5-lilypad/fcitx5-lilypad

# 2. Biên dịch C++ Addon & Server Daemon
mkdir -p build && cd build
cmake -DCMAKE_INSTALL_PREFIX=/usr ..
make -j$(nproc)
sudo make install
```

#### Bước 2: Nạp Uinput & Kích hoạt Daemon Service

```bash
# 3. Kiểm tra uinput module, reload systemd, user proxy & udev rules (/dev/uinput)
ls /dev/uinput || sudo modprobe uinput
sudo systemd-sysusers
sudo udevadm control --reload-rules && sudo udevadm trigger
sudo systemctl daemon-reload

# 4. Kích hoạt Server Daemon qua Systemd
sudo systemctl enable --now fcitx5-lilypad-server@$USER.service

# 5. Khởi động lại Fcitx5
fcitx5 -r -d
```

---

## 🤝 Kêu Gọi Cộng Đồng Đóng Góp (Call for Testers)

Nếu bạn đang sử dụng **Hyprland, Sway, KDE, GNOME, Fedora, Ubuntu, NixOS, Void...**, hãy giúp bộ gõ hoàn thiện hơn bằng cách:

1. **Thử nghiệm bộ gõ** trên các ứng dụng bạn hay dùng hàng ngày (Chrome, Discord, Telegram, VS Code, LibreOffice, Facebook Web).
2. **Báo cáo sự cố (Issue)** nếu gặp lỗi lặp chữ, đơ chữ hoặc nuốt ký tự.
3. **Gửi Log thời gian thực**: Sử dụng script [scripts/read_logs.sh](file:///home/chiconcota/Documents/vnlilypad-lotus/scripts/read_logs.sh) để đính kèm log chi tiết khi báo lỗi.
4. **Đóng góp Code (Pull Request)**: Viết thêm Cảm biến ACK chuyên biệt (`IAckSensor`) cho các compositor cụ thể.

---

## 📄 Cấu Trúc Dự Án

- Mã nguồn C++ Fcitx5 Addon: [fcitx5-lilypad/](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/)
- Server Daemon Uinput: [fcitx5-lilypad/server/](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/server/)
- Cảm biến ACK & IKI: [fcitx5-lilypad/src/ack-sensors/](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/ack-sensors/) & [fcitx5-lilypad/src/iki-sensors/](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/iki-sensors/)
- Tài liệu Kiến trúc Hệ thống: [.fcitx5-lilypad-ai/](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/)
- Script đọc Log thời gian thực: [scripts/read_logs.sh](file:///home/chiconcota/Documents/vnlilypad-lotus/scripts/read_logs.sh)

---

## 📄 Giấy Phép (License)

Dự án được phân phối dưới Giấy phép **GPL-3.0-or-later**. Xem [LICENSE](LICENSE) để biết thêm chi tiết.
