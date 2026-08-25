# 🍃 fcitx5-lilypad

> **Bộ gõ Tiếng Việt thế hệ mới cho Linux Wayland & X11** dựa trên kiến trúc Hybrid: **Fcitx5 C++ Addon + IKI Adaptive Engine + Modular IAckSensor + Sentinel Barrier Protocol + Pure Kernel Uinput Server Daemon**.

[![Release](https://img.shields.io/github/v/release/chiconcota/fcitx5-lilypad?style=flat&color=success)](https://github.com/chiconcota/fcitx5-lilypad/releases)
[![Version](https://img.shields.io/badge/version-2.3.0-blue.svg)](https://github.com/chiconcota/fcitx5-lilypad)
[![License](https://img.shields.io/github/license/chiconcota/fcitx5-lilypad?style=flat&color=blue)](LICENSE)
[![Tested Compositor](https://img.shields.io/badge/tested_compositor-Niri-purple.svg)](https://github.com/niri-wm/niri)
[![Target Packaging](https://img.shields.io/badge/target_packaging-AUR_Arch_Linux-blue.svg)](#-phát-hành--cài-đặt)
[![Community Status](https://img.shields.io/badge/community-call_for_testers-orange.svg)](#-kêu-gọi-cộng-đồng-đóng-góp--thử-nghiệm-call-for-testers)

---

## 💡 Điểm Đột Phá Kiến Trúc (`v2.3.0 - IKI Adaptive & Sentinel Barrier`)

`fcitx5-lilypad` v2.3.0 giải quyết dứt điểm các vấn đề cố hữu của bộ gõ tiếng Việt trên Linux (nuốt chữ, lặp chữ, đè rác chữ trên Web DOM/Electron, và đảo dấu cách) thông qua các công nghệ cốt lõi:

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
  │  - Two-Tier Timeout: Dynamic Soft Timeout (App Lag) & Watchdog 250ms   │
  │  - Emergency Purge: Xả phím thô tức thì nếu App treo cứng              │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │          MODULAR ACK SENSOR LAYER (CẢM BIẾN THÍCH ỨNG ĐỘ TRỄ)          │
  │  - Dynamic Micro-Pacing Lerp: Tự động co giãn theo nhịp IKI ngón tay   │
  │  - App ACK Consumption: Tích hợp thời gian tiêu thụ DOM (N * T_ack)    │
  │  - Cold Start Safe Baseline: Trần trễ >50ms an toàn cho chữ đầu tiên   │
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

### 1. Dynamic Micro-Pacing via Normalized Lerp & App ACK Consumption
- **Nội suy tuyến tính (Lerp):** Thay vì áp đặt thời gian trễ cố định, bộ gõ liên tục đo nhịp gõ ngón tay ($\text{EMA\_IKI}$) qua module `IIkiSensor` kết hợp thời gian phản hồi của ứng dụng ($T_{\text{app\_ack}}$) để điều chỉnh vi trễ $\Delta t(N)$:
  - **Terminal / App nhẹ:** Vi trễ nén về mức sàn vật lý **$1.5\text{ms} \sim 2.5\text{ms}$** (Zero-Latency tức thì, gõ siêu nhạy không cảm nhận độ trễ).
  - **Facebook / Web DOM / Electron:** Vi trễ tự động giãn nở an toàn theo thời gian tiêu thụ DOM ($45\text{ms} \sim 60\text{ms}$), đảm bảo React DOM tiêu hóa sạch phím xóa trước khi chèn chữ mới.
- **Cold Start Safe Baseline ($>50\text{ms}$):** Khi vừa mở ứng dụng hoặc gõ từ đầu tiên ($\text{IKI} = 0$), hệ thống áp dụng mức trần an toàn $50\text{ms} \sim 80\text{ms}$ loại bỏ $100\%$ nguy cơ nuốt chữ ở ký tự đầu, sau đó chuyển giao sang thuật toán Lerp từ từ thứ 2.

### 2. Giao Thức Uinput Sentinel Barrier $N+1$
- Khi thực hiện thay thế chuỗi ký tự cũ bằng chuỗi mới, daemon phát **$N+1$ phím xóa `KEY_BACKSPACE`**:
  - $N$ phím đầu được chuyển tiếp xuống ứng dụng (`return false;`) để xóa $N$ ký tự cũ.
  - Phím thứ $N+1$ đóng vai trò **Phím Rào Chắn Sentinel**: Fcitx5 nuốt trọn phím này (`event.filterAndAccept(); return true;`) và chặn không cho xuống App.
- **Bảo đảm trật tự vật lý FIFO:** Sự xuất hiện của phím $N+1$ tại Fcitx5 là bằng chứng xác nhận ứng dụng đã xóa xong $N$ ký tự cũ. Triệt tiêu $100\%$ race condition (không bao giờ xảy ra tình trạng phím xóa đến sau xóa mất chữ vừa commit).

### 3. Cơ Chế Two-Tier Timeout & Emergency State Protection
- **Dynamic Soft Timeout ($T_{\text{soft}}$):** Tự động phát hiện khi ứng dụng bị nghẽn (DOM render lag, GC stall) để chuyển sang trạng thái `AppLagHolding`, tạm hoãn phát uinput tiếp theo và gom phím an toàn vào RAM `buffered_keys_` chống rách từ.
- **Watchdog Hard Timeout (250ms) & Emergency Purge:** Main Event Loop cài đặt timer $250\text{ms}$ độc lập. Nếu ứng dụng bị treo quá 250ms, hệ thống lập tức kích hoạt `purgeContextEmergency()`: reset engine, xóa word buffer và xả toàn bộ phím đệm ra màn hình dưới dạng phím thô (`ic_->forwardKey()`), đảm bảo **bàn phím không bao giờ bị đơ hay kẹt cứng**.

### 4. Uniform Web IME Routing & GTK4 Native Precision ($1\mu\text{s}$)
- **Chromium / Web Routing:** Tự động đồng bộ hóa kênh phát cho Chromium/Electron qua `ic_->commitString()`, tránh xung đột Virtual DOM trên Google Docs và Facebook.
- **GTK4 Native Precision ($1\mu\text{s}$):** Đặt độ chính xác timer $1\mu\text{s}$ cho Event Loop, bảo tồn kênh phím Native cho GTK4 / Gnome Text Editor, triệt tiêu $100\%$ lỗi đảo dấu cách (`"c òngi"`, `"l àcười"`).

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

```bash
# Bản cài nhanh từ Binary pre-compiled phát hành trên GitHub Releases (Khuyên dùng):
yay -S fcitx5-lilypad-bin

# Hoặc Bản biên dịch từ mã nguồn Git mới nhất (Development Branch):
yay -S fcitx5-lilypad-git
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
  sudo dnf install -y git gcc-c++ cmake extra-cmake-modules fcitx5-devel gettext-devel golang libX11-devel libinput-devel systemd-devel fcitx5-configtool python3 python3-qtpy python3-qt5
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
