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
    <br />
    <a href="https://github.com/chiconcota/fcitx5-lilypad/releases">
      <img src="https://img.shields.io/github/v/release/chiconcota/fcitx5-lilypad?style=flat&color=success" alt="Release">
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

## 🚀 Định Hướng Đóng Gói Lên AUR (Arch User Repository)

Bộ gõ **`fcitx5-lilypad`** đang được chuẩn bị hoàn thiện để phát hành chính thức gói **AUR (`fcitx5-lilypad`)** dành cho người dùng Arch Linux và các distro biến thể (Manjaro, EndeavourOS, Garuda Linux).

---

## 🎯 Trạng Thái Hiện Tại & Môi Trường Thử Nghiệm

Hiện tại, dự án được tác giả **tối ưu hóa và kiểm thử trực tiếp trên Niri Compositor (Arch Linux)** với module cảm biến thích ứng `NiriAckSensor`. 

Do giới hạn về thiết bị thử nghiệm, tác giả **chưa thể kiểm thử toàn diện trên tất cả các Window Compositors và Linux Distros khác nhau**. Vì vậy, **sự đóng góp, thử nghiệm và phản hồi từ Cộng đồng Linux Việt Nam là cực kỳ quan trọng!**

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

## 📦 Cài Đặt & Hướng Dẫn Sử Dụng

### Biên dịch từ Mã Nguồn (Build from Source)

```bash
# 1. Clone repository
git clone https://github.com/chiconcota/fcitx5-lilypad.git
cd fcitx5-lilypad

# 2. Biên dịch C++ Addon
mkdir build && cd build
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

## 📄 Giấy Phép (License)

Phân phối dưới Giấy phép GPL-3.0-or-later. Xem `LICENSE` để biết thêm chi tiết.
