# 🍃 fcitx5-lilypad

> **Bộ gõ Tiếng Việt thế hệ mới cho Linux Wayland & X11** dựa trên kiến trúc Hybrid: **Fcitx5 C++ Addon + Modular IAckSensor + Pure Kernel Uinput Server Daemon**.

[![Release](https://img.shields.io/github/v/release/chiconcota/fcitx5-lilypad?style=flat&color=success)](https://github.com/chiconcota/fcitx5-lilypad/releases)
[![License](https://img.shields.io/github/license/chiconcota/fcitx5-lilypad?style=flat&color=blue)](LICENSE)
[![Tested Compositor](https://img.shields.io/badge/tested_compositor-Niri-purple.svg)](https://github.com/niri-wm/niri)
[![Target Packaging](https://img.shields.io/badge/target_packaging-AUR_Arch_Linux-blue.svg)](#-phát-hành--cài-đặt)
[![Community Status](https://img.shields.io/badge/community-call_for_testers-orange.svg)](#-kêu-gọi-cộng-đồng-đóng-góp--thử-nghiệm-call-for-testers)

---

## 🚀 Định Hướng Đóng Gói Lên AUR (Arch User Repository)

Bộ gõ **`fcitx5-lilypad`** đang được chuẩn bị hoàn thiện để phát hành chính thức gói **AUR (`fcitx5-lilypad`)** dành cho người dùng Arch Linux và các distro biến thể (Manjaro, EndeavourOS, Garuda Linux).

---

## 🎯 Trạng Thái Hiện Tại & Môi Trường Thử Nghiệm

Hiện tại, dự án được tác giả **tối ưu hóa và kiểm thử trực tiếp trên Niri Compositor (Arch Linux)** với module cảm biến thích ứng `NiriAckSensor`. 

Do giới hạn về phần cứng và thiết bị thử nghiệm, tác giả **chưa thể kiểm thử toàn diện trên tất cả các Window Compositors và Linux Distros khác nhau**. Vì vậy, **sự đóng góp, thử nghiệm và phản hồi từ Cộng đồng Linux Việt Nam là cực kỳ quan trọng!**

### 📊 Ma Trận Tương Thích (Compositor & Distro Matrix)

| Window Compositor | Trạng thái | Ghi chú | Nhu cầu đóng góp |
| :--- | :---: | :--- | :---: |
| **Niri** | 🟢 **Sẵn sàng** | Tích hợp `NiriAckSensor` + EMA Machine Learning Control | Thử nghiệm nâng cao |
| **Hyprland** | 🟡 **Thử nghiệm** | Chạy vạn năng qua `GenericAckSensor` | **Cần Tester & Maintainer** |
| **Sway** | 🟡 **Thử nghiệm** | Hỗ trợ qua Wayland `zwp_input_method_v1/v2` | **Cần Tester** |
| **KDE Plasma (Wayland)**| 🟡 **Thử nghiệm** | Cần kiểm thử độ trễ IPC Wayland | **Cần Tester** |
| **GNOME (Wayland)** | 🟡 **Thử nghiệm** | Cần kiểm thử tương thích với Mutter | **Cần Tester** |
| **X11 (Generic)** | 🟡 **Thử nghiệm** | Fallback qua Fcitx5 X11 Frontend + `GenericAckSensor` (không có Wayland Protocols) | **Cần Tester** |

---

## 🤝 Kêu Gọi Cộng Đồng Đóng Góp (Call for Testers)

Nếu bạn đang sử dụng **Hyprland, Sway, KDE, GNOME, Fedora, Ubuntu, NixOS, Void...**, hãy giúp bộ gõ hoàn thiện hơn bằng cách:

1. **Thử nghiệm bộ gõ** trên môi trường của bạn.
2. **Báo cáo sự cố (Issue)** nếu gặp lỗi lặp chữ, đơ chữ hoặc kẹt bộ đệm.
3. **Gửi Log thời gian thực**: Sử dụng script [scripts/read_logs.sh](file:///home/chiconcota/Documents/vnlilypad-lotus/scripts/read_logs.sh) để đính kèm log chi tiết khi báo lỗi.
4. **Đóng góp Code (Pull Request)**: Viết thêm Cảm biến ACK riêng (`IAckSensor`) cho các compositor như Hyprland hay Sway.

---

## 💡 Điểm Nổi Bật Của Kiến Trúc Mới (`v2.2.0-modular-sensor`)

1. **Chế Độ Gõ `Sequence` (ID 9) & Modular IAckSensor Architecture:**
   - **Tích hợp Sequencer Layer C++:** Duy trì máy trạng thái vi bước nguyên tử (Micro-Step State Machine), đếm token xóa nguyên tử `expected_swallow_backspaces_`, và tự động đo độ trễ `elapsed` thực tế của ứng dụng.
   - **EMA Machine Learning Control:** Tự động nới rộng Dynamic Barrier khi App/Messenger bị lag DOM và tự suy giảm (decay) nhanh về $5\text{ms}$ khi App mượt.

2. **Optimized Batch Replay Protocol:**
   - Xả hàng đợi phím đệm tốc độ $0.1\text{ms}$ cho ký tự thường và hoãn nhịp $3\text{ms}$ cho phím `Space`, bẻ gãy 100% bẫy đệ quy hoãn 15ms từng phím, triệt hạ hoàn toàn lỗi kẹt phím 4.5s khi gõ tốc độ cao.

3. **Multi-User Systemd Daemon (`fcitx5-lilypad-server@.service`):**
   - Chạy dịch vụ daemon uinput độc lập per-user, tự động xác thực IPC bằng `SO_PEERCRED` (`cred.uid == expected_uid`), đảm bảo an toàn tuyệt đối cho hệ thống multi-user.

---

## 📦 Phát Hành & Cài Đặt (Installation Guide)

### 1. Biên dịch và Cài đặt từ Mã Nguồn (Build from Source)

```bash
# 0. Cài đặt Fcitx5 và các gói phụ thuộc biên dịch (Dependencies)
# Ubuntu / Debian:
sudo apt update && sudo apt install -y git fcitx5 fcitx5-config-qt libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev fcitx5-modules-dev libinput-dev libudev-dev extra-cmake-modules build-essential cmake gettext golang libx11-dev

# Arch Linux:
# sudo pacman -S --needed git base-devel cmake extra-cmake-modules fcitx5 fcitx5-configtool fcitx5-qt fcitx5-gtk gettext fmt go

# Fedora:
# sudo dnf install -y git gcc-c++ cmake extra-cmake-modules fcitx5-devel gettext-devel golang libX11-devel

# 1. Clone repository (kèm --recursive để tự động nạp submodule bamboo-core)
git clone --recursive https://github.com/chiconcota/fcitx5-lilypad.git
cd fcitx5-lilypad/fcitx5-lilypad

# 2. Biên dịch C++ Addon
mkdir -p build && cd build
cmake -DCMAKE_INSTALL_PREFIX=/usr ..
make -j$(nproc)
sudo make install

# 3. Kiểm tra uinput module, khởi tạo user proxy & reload udev rules (/dev/uinput)
ls /dev/uinput || sudo modprobe uinput
sudo systemd-sysusers
sudo udevadm control --reload-rules && sudo udevadm trigger

# 4. Kích hoạt Server Daemon qua Systemd
sudo systemctl enable --now fcitx5-lilypad-server@$USER.service

# 5. Khởi động lại Fcitx5
fcitx5 -r -d
```

---

## 📄 Thư Mục Dự Án Chi Tiết

- Mã nguồn C++ Fcitx5 Addon: [fcitx5-lilypad/](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/)
- Tài liệu Kiến trúc Hệ thống: [.fcitx5-lilypad-ai/](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/)
- Script đọc Log thời gian thực: [scripts/read_logs.sh](file:///home/chiconcota/Documents/vnlilypad-lotus/scripts/read_logs.sh)
