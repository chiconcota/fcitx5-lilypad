[English](README.en.md) | [Tiếng Việt](README.md)

<a id="readme-top"></a>

<div align="center">
  <a href="https://github.com/chiconcota/fcitx5-lilypad">
    <img src="data/fcitx-lilypad-README.svg" alt="Logo" width="80" height="80">
  </a>

<h2 align="center">Fcitx5 Lilypad</h2>

<p align="center">
    <b>Bộ gõ tiếng Việt thế hệ mới cho Linux Wayland & X11</b>
    <br />
    <i>Kiến trúc Hybrid: Fcitx5 C++ Addon + IKI Adaptive Engine + Sentinel Barrier Protocol + Uinput Server Daemon</i>
    <br />
    <br />
    <a href="https://github.com/chiconcota/fcitx5-lilypad/releases">
      <img src="https://img.shields.io/github/v/release/chiconcota/fcitx5-lilypad?style=flat&color=success" alt="Release">
    </a>
    <a href="https://github.com/chiconcota/fcitx5-lilypad">
      <img src="https://img.shields.io/badge/version-2.3.1-blue.svg" alt="Version 2.3.1">
    </a>
    <a href="https://github.com/chiconcota/fcitx5-lilypad/blob/main/LICENSE">
      <img src="https://img.shields.io/github/license/chiconcota/fcitx5-lilypad?style=flat&color=blue" alt="License">
    </a>
    <a href="https://github.com/niri-wm/niri">
      <img src="https://img.shields.io/badge/tested_compositor-Niri-purple.svg" alt="Niri Tested">
    </a>
    <a href="#cài-đặt--hướng-dẫn-sử-dụng">
      <img src="https://img.shields.io/badge/target_packaging-AUR_Arch_Linux-blue.svg" alt="AUR Target">
    </a>
    <a href="https://github.com/chiconcota/fcitx5-lilypad/issues">
      <img src="https://img.shields.io/github/issues/chiconcota/fcitx5-lilypad?style=flat&color=red" alt="Issues">
    </a>
  </p>

<p align="center">
    <a href="#cài-đặt--hướng-dẫn-sử-dụng"><strong>Cài đặt »</strong></a>
    ·
    <a href="https://github.com/chiconcota/fcitx5-lilypad/issues/new">Báo lỗi</a>
    ·
    <a href="https://github.com/chiconcota/fcitx5-lilypad/issues/new">Yêu cầu tính năng</a>
  </p>
</div>

<br />

Dự án này là bản nâng cấp tối ưu hóa kiến trúc dựa trên [VMK](https://github.com/thanhpy2009/VMK). Chân thành cảm ơn tác giả Thành đã đặt nền móng cho bộ gõ này.

---

## 💡 Điểm Đột Phá Kiến Trúc (`v2.3.0 - IKI Adaptive & Sentinel Barrier`)

`fcitx5-lilypad` v2.3.0 là giải pháp toàn diện loại bỏ triệt để các hạn chế cố hữu của bộ gõ Tiếng Việt trên Wayland/X11:

### 1. Dynamic Micro-Pacing via Normalized Lerp & App ACK Consumption
- **Module Cảm Biến Nhịp Tay (`IIkiSensor`):** Đo liên tục khoảng cách thời gian giữa các lần gõ phím vật lý ($\mathrm{EMA}_{\mathrm{IKI}}$).
- **Nội Suy Tuyến Tính (Lerp) Theo Tốc Độ Tiêu Thụ Của Ứng Dụng ($N \times T_{\text{ack}}$):**
  - **Trên Terminal / App nhẹ:** Vi trễ nén về mức sàn vật lý **$1.5\text{ms} \sim 2.5\text{ms}$** (Zero-Latency tức thì, gõ siêu nhạy).
  - **Trên Facebook / Web DOM:** Vi trễ tự động dãn nở an toàn theo thời gian tiêu thụ DOM ($45\text{ms} \sim 60\text{ms}$), chống đè rác chữ và nuốt chữ.
- **Cold Start Safe Baseline ($>50\text{ms}$):** Khi gõ từ đầu tiên lúc chưa có dữ liệu lịch sử $\text{IKI}$ và $\text{App ACK}$, hệ thống áp dụng ngưỡng an toàn $50\text{ms} \sim 80\text{ms}$ bảo đảm 100% không nuốt chữ.

### 2. Giao Thức Uinput Sentinel Barrier $N+1$
- Khi xóa $N$ ký tự cũ, daemon bắn $N+1$ phím xóa `KEY_BACKSPACE` qua `/dev/uinput`:
  - $N$ phím đầu xóa sạch text cũ trong ứng dụng.
  - Phím thứ $N+1$ được Fcitx5 nuốt trọn (`filterAndAccept`) làm chốt chặn an toàn (Sentinel).
- **Trật tự vật lý FIFO:** Sự xuất hiện của phím $N+1$ tại Fcitx5 là bằng chứng phần cứng xác nhận $N$ phím trước đã vào App xong, triệt tiêu $100\%$ xung đột phím xóa nhầm chuỗi vừa commit.

### 3. Two-Tier Timeout & Emergency State Protection
- **Dynamic Soft Timeout ($T_{\text{soft}}$):** Khi App bị giật/lag DOM, Sequencer chuyển sang `BarrierState::AppLagHolding` và gom phím an toàn vào RAM `buffered_keys_` chống rách từ.
- **Watchdog Hard Timeout (250ms) & Emergency Purge:** Main Event Loop cài đặt timer 250ms độc lập. Nếu ứng dụng treo quá 250ms, hệ thống tự động kích hoạt `purgeContextEmergency()` xả toàn bộ phím thô an toàn, **không bao giờ đơ/kẹt bàn phím**.

### 4. Uniform Web IME Routing & GTK4 Native Precision ($1\mu\text{s}$)
- Tự động đồng bộ luồng commit cho Chromium/Electron chống xung đột Virtual DOM.
- Giữ nguyên kênh phím Native với độ chính xác $1\mu\text{s}$ cho GTK4 / Text Editor, triệt tiêu lỗi đảo dấu cách.

---

## 🎯 Trạng Thái Hiện Tại & Môi Trường Thử Nghiệm

| Window Compositor | Trạng thái | Ghi chú | Nhu cầu đóng góp |
| :--- | :---: | :--- | :---: |
| **Niri** | 🟢 **Sẵn sàng** | Tích hợp `NiriAckSensor` + IKI Adaptive + Sentinel Barrier | Kiểm thử liên tục |
| **Hyprland** | 🟢 **Sẵn sàng** | Chạy vạn năng qua `GenericAckSensor` | Tester & Feedback |
| **Sway** | 🟢 **Sẵn sàng** | Hỗ trợ qua Wayland `zwp_input_method_v1/v2` | Tester & Feedback |
| **KDE Plasma (Wayland)**| 🟢 **Sẵn sàng** | Chạy qua KWin Wayland IME API | Tester & Feedback |
| **GNOME (Wayland)** | 🟢 **Sẵn sàng** | Chạy qua Mutter Wayland Input API | Tester & Feedback |
| **X11 (Generic)** | 🟢 **Sẵn sàng** | Fallback qua Fcitx5 X11 Frontend + `GenericAckSensor` | Tester & Feedback |

---

## 📦 Cài Đặt & Hướng Dẫn Sử Dụng

### Cài đặt qua AUR (Arch Linux / Manjaro / EndeavourOS)

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

### Biên dịch từ Mã Nguồn (Build from Source)

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

# 2. Biên dịch C++ Addon
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

1. **Thử nghiệm bộ gõ** trên môi trường của bạn.
2. **Báo cáo sự cố (Issue)** nếu gặp lỗi lặp chữ, đơ chữ hoặc kẹt bộ đệm.
3. **Gửi Log thời gian thực**: Sử dụng script [scripts/read_logs.sh](file:///home/chiconcota/Documents/vnlilypad-lotus/scripts/read_logs.sh) để đính kèm log chi tiết khi báo lỗi.
4. **Đóng góp Code (Pull Request)**: Viết thêm Cảm biến ACK riêng (`IAckSensor`) cho các compositor chuyên biệt.

---

## 📄 Giấy Phép (License)

Phân phối dưới Giấy phép **GPL-3.0-or-later**. Xem `LICENSE` để biết thêm chi tiết.
