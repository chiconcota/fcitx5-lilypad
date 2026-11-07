[English](README.en.md) | [Tiếng Việt](README.md)

<a id="readme-top"></a>

<div align="center">
  <a href="https://github.com/chiconcota/fcitx5-lilypad">
    <img src="data/fcitx-lilypad-README.svg" alt="Logo" width="96" height="96">
  </a>

  <h2>🍃 Fcitx5 Lilypad</h2>

  <p>
    <b>Bộ gõ Tiếng Việt thế hệ mới cho Linux Wayland & X11</b>
    <br />
    <i>Siêu tốc • Không nuốt chữ • Triệt tiêu đơ phím • Tự động thích ứng Electron/Wayland</i>
  </p>

  <p>
    <a href="https://github.com/chiconcota/fcitx5-lilypad/releases">
      <img src="https://img.shields.io/github/v/release/chiconcota/fcitx5-lilypad?style=flat&color=success" alt="Release">
    </a>
    <a href="https://github.com/chiconcota/fcitx5-lilypad">
      <img src="https://img.shields.io/badge/version-2.5.1-blue.svg" alt="Version 2.5.1">
    </a>
    <a href="https://github.com/chiconcota/fcitx5-lilypad/blob/main/LICENSE">
      <img src="https://img.shields.io/github/license/chiconcota/fcitx5-lilypad?style=flat&color=blue" alt="License">
    </a>
    <a href="https://github.com/niri-wm/niri">
      <img src="https://img.shields.io/badge/Wayland-Niri%20%7C%20Hyprland%20%7C%20GNOME%20%7C%20KDE-purple.svg" alt="Wayland Tested">
    </a>
    <a href="#-hướng-dẫn-cài-đặt-chi-tiết-theo-từng-distro">
      <img src="https://img.shields.io/badge/Distros-Arch%20%7C%20Ubuntu%20%7C%20Fedora-blue.svg" alt="Distros">
    </a>
  </p>

  <p>
    <a href="#-giới-thiệu-tổng-quan"><strong>Giới thiệu</strong></a> ·
    <a href="#-tính-năng-nổi-bật"><strong>Tính năng</strong></a> ·
    <a href="#-hướng-dẫn-cài-đặt-chi-tiết-theo-từng-distro"><strong>Cài đặt</strong></a> ·
    <a href="#-kích-hoạt--sử-dụng"><strong>Kích hoạt</strong></a> ·
    <a href="#-lời-cảm-ơn-credits"><strong>Tác giả & Tri ân</strong></a>
  </p>
</div>

---

## 🌟 Giới Thiệu Tổng Quan

**Fcitx5 Lilypad** là bộ gõ tiếng Việt hiện đại được thiết kế từ đầu dành riêng cho hệ điều hành Linux (hỗ trợ hoàn hảo cả Wayland thế hệ mới lẫn X11 truyền thống). 

Nếu bạn từng cảm thấy bực bội vì các vấn đề cố hữu của bộ gõ tiếng Việt trên Linux như:
- ❌ **Nuốt chữ, mất chữ âm ghép** khi gõ nhanh trên trình duyệt và các ứng dụng Electron (`thương` $\to$ `tương`, `thành` $\to$ `th`).
- ❌ **Đơ, kẹt bàn phím** khi ứng dụng bị khựng hoặc giật lag.
- ❌ **Bung phím số thô** khi gõ VNI (`co6 thuo7g`) hoặc lặp ký tự trong Telex (`nuuo7n rẫy`) trên các ứng dụng phức tạp (AFFiNE, Notion, Antigravity IDE, VS Code).
- ❌ **Đảo dấu cách, mất chữ** khi gõ cạnh dấu câu hoặc sau khi dán văn bản (Paste).

👉 **Fcitx5 Lilypad sinh ra để giải quyết triệt để tất cả các vấn đề trên!** Bằng cách kết hợp lõi xử lý âm tiết **Bamboo Core (Rust Native)** cùng kiến trúc **Micro-Step Sequencer** và **Dynamic Adaptive Governor**, Lilypad đem lại trải nghiệm gõ phím tiếng Việt mượt mà, chính xác và tự nhiên như đang gõ trên bàn phím phần cứng.

---

## 🚀 Tính Năng Nổi Bật

### 🦀 1. Lõi Âm Tiết Bamboo Native Core (Thuần Rust C-ABI)
- Tích hợp trực tiếp engine [bamboo-core](https://github.com/bamboo-vi/bamboo-core) viết bằng Rust nguyên bản, an toàn bộ nhớ tuyệt đối (Zero Memory Leak, Zero Garbage Collection).
- Hỗ trợ đầy đủ các kiểu gõ thông dụng: **Telex, VNI, VIQR, Microsoft Layout, Telex 2, Telex W, VNI Bàn phím tiếng Pháp**.
- Hỗ trợ đa dạng bảng mã: **Unicode chuẩn, TCVN3 (ABC), VNI Windows, VIQR**.
- Cơ chế khôi phục từ sai chính tả thông minh (**Smart Auto-Restore**) và tự do đặt dấu thanh (**Free Marking**).

### ⚡ 2. Kiến Trúc Sequencer & Hardware Sentinel (`KEY_PAUSE`)
- **Nguyên tử hóa vi bước (Atomic Micro-Steps):** Tách bạch rõ ràng giữa vi bước phát phím xóa (Backspace) và chèn chuỗi ký tự mới (Commit).
- **Phím chốt chặn phần cứng phi phá hủy (`KEY_PAUSE`):** Sử dụng mã quét phần cứng tiêu chuẩn làm rào cản an toàn FIFO, loại bỏ $100\%$ race condition mà không bao giờ gây nguy cơ xóa lẹm văn bản trước đó của người dùng.
- **Watchdog Hard Timeout 250ms:** Luôn bảo đảm bàn phím không bao giờ bị đơ hay treo cứng; tự động xả phím an toàn khi ứng dụng đích bị đóng băng.

### 🧠 3. Bộ Điều Phối Trễ Thích Ứng (Dynamic Adaptive Governor)
- **Terminal & Native GUI (Alacritty, Kitty, Foot, Kate, Text Editor):** Sàn trễ Pre-commit siêu tốc **$1.5\text{ms}$** ($1500\mu\text{s}$) và Post-commit **$0.3\text{ms}$** ($300\mu\text{s}$), cho tốc độ phản hồi tức thì với độ trễ gần như bằng 0.
- **Chromium & Electron (Chrome, Brave, Edge, VS Code, Discord, Slack, AFFiNE):** Tự động đồng bộ chuẩn nhịp **1 khung hình VSync ($18\text{ms} \sim 22\text{ms}$)** (chuẩn an toàn của Lotus). Cung cấp đủ thời gian cho Virtual DOM của Electron hoàn tất chu kỳ xóa con trỏ trước khi ký tự mới xuất hiện, chấm dứt hoàn toàn hiện tượng nuốt chữ hay bung phím thô VNI.

### 🛡️ 4. Virtual Focus Pinning & Focus Bounce Protection
- Tự động nhận diện và bảo vệ đệm từ dở dang khi các ứng dụng web phức tạp (như AFFiNE Canvas, Notion, Antigravity IDE) liên tục phát các sự kiện FocusOut/FocusIn giả lập do cập nhật DOM.
- Tích hợp IPC trực tiếp qua Socket của Compositor (`$NIRI_SOCKET`) truy vấn `app_id` trong $155\mu\text{s}$, nhận diện chuẩn xác các ứng dụng Wayland chạy trong container / iframe.

### 🎮 5. Chế Độ Gõ Đa Dạng Theo Nhu Cầu
- **Sequence Mode (Khuyên dùng):** Phát phím xóa qua Linux Kernel `/dev/uinput` riêng biệt, gõ mượt mà trực tiếp trong văn bản như Unikey trên Windows.
- **Preedit Mode:** Chế độ hiển thị thanh gạch chân truyền thống của Fcitx5, tương thích tối đa trên mọi ứng dụng lạ.
- **Minecraft Mode:** Tối ưu hóa phím xóa chuyên biệt cho Minecraft và các tựa game full-screen.

---

## 📦 Hướng Dẫn Cài Đặt Chi Tiết Theo Từng Distro

### 1. Arch Linux / Manjaro / CachyOS / EndeavourOS

Người dùng Arch Linux có thể cài đặt trực tiếp từ AUR:

```bash
# Cách 1: Cài bản binary chính thức (Khuyên dùng - Cài ngay không cần build):
yay -S fcitx5-lilypad-bin

# Cách 2: Cài bản tự biên dịch từ mã nguồn phát hành:
yay -S fcitx5-lilypad

# Cách 3: Cài bản git phát triển mới nhất:
yay -S fcitx5-lilypad-git
```

Hoặc biên dịch thủ công:
```bash
sudo pacman -S --needed base-devel cmake extra-cmake-modules ninja fcitx5 fcitx5-configtool git rust libx11 libinput systemd acl python python-qtpy python-pyqt5
git clone https://github.com/chiconcota/fcitx5-lilypad.git
cd fcitx5-lilypad/fcitx5-lilypad
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -GNinja
cmake --build build -j$(nproc)
sudo cmake --install build
```

---

### 2. Ubuntu / Debian / Linux Mint / Pop!_OS

#### Bước 1: Cài đặt các gói phụ thuộc (Dependencies)
```bash
sudo apt update
sudo apt install -y git build-essential cmake extra-cmake-modules ninja-build \
    gettext pkg-config libx11-dev libfcitx5core-dev libfcitx5utils-dev \
    libfcitx5config-dev fcitx5-modules-dev libinput-dev libudev-dev \
    cargo rustc acl python3 python3-qtpy python3-pyqt5 fcitx5 fcitx5-config-qt
```

#### Bước 2: Tải mã nguồn và biên dịch
```bash
git clone https://github.com/chiconcota/fcitx5-lilypad.git
cd fcitx5-lilypad/fcitx5-lilypad
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -GNinja
cmake --build build -j$(nproc)
sudo cmake --install build
```

---

### 3. Fedora / RHEL / AlmaLinux

#### Bước 1: Cài đặt các gói phụ thuộc (Dependencies)
```bash
sudo dnf install -y git gcc-c++ cmake extra-cmake-modules ninja-build \
    gettext pkgconf-pkg-config libX11-devel fcitx5-devel libinput-devel \
    systemd-devel cargo rust acl python3 python3-QtPy python3-qt5 \
    fcitx5 fcitx5-configtool fcitx5-autostart
```

#### Bước 2: Tải mã nguồn và biên dịch
```bash
git clone https://github.com/chiconcota/fcitx5-lilypad.git
cd fcitx5-lilypad/fcitx5-lilypad
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -GNinja
cmake --build build -j$(nproc)
sudo cmake --install build
```

---

## ⚙️ Kích Hoạt & Cấu Hình Sau Khi Cài Đặt

Chế độ gõ **Sequence Mode** sử dụng dịch vụ daemon `fcitx5-lilypad-server` để phát phím xóa vi mô qua `/dev/uinput`. Bạn cần kích hoạt dịch vụ 1 lần duy nhất:

### Bước 1: Kích hoạt quyền Uinput và Service Daemon (Áp dụng cho mọi Distro)
```bash
# 1. Đảm bảo uinput kernel module được nạp
sudo modprobe uinput

# 2. Cập nhật quyền udev rules và nạp lại systemd
sudo udevadm control --reload-rules && sudo udevadm trigger
sudo systemctl daemon-reload

# 3. Kích hoạt Server Daemon chạy ngầm theo user của bạn:
sudo systemctl enable --now fcitx5-lilypad-server@$USER.service

# 4. Khởi động lại Fcitx5:
fcitx5 -r -d
```

### Bước 2: Thêm Lilypad vào danh sách bộ gõ Fcitx5
1. Mở công cụ cấu hình Fcitx5:
   ```bash
   fcitx5-configtool
   ```
2. Bỏ tích ô *"Only Show Current Language"* (Chỉ hiển thị ngôn ngữ hiện tại).
3. Tìm kiếm **`Lilypad`** và bấm nút mũi tên sang phải `>` để thêm vào danh sách Input Method.
4. Đặt Lilypad dưới bàn phím tiếng Anh mặc định (ví dụ: `Keyboard - English (US)`).
5. Sử dụng phím tắt chuyển bộ gõ mặc định (thường là `Ctrl + Space` hoặc `Super + Space`) để bắt đầu gõ tiếng Việt!

---

## 📊 Ma Trận Tương Thích (Desktop & Compositor)

| Môi trường / Compositor | Giao thức | Trạng thái | Ghi chú |
| :--- | :---: | :---: | :--- |
| **Niri** | Wayland | 🟢 Hoàn hảo | Hỗ trợ chuyên sâu qua `NiriAckSensor` + IPC Socket |
| **Hyprland** | Wayland | 🟢 Hoàn hảo | Chạy mượt mà qua `GenericAckSensor` |
| **Sway** | Wayland | 🟢 Hoàn hảo | Tích hợp qua `zwp_input_method_v2` |
| **KDE Plasma (Wayland / X11)** | Wayland / X11 | 🟢 Hoàn hảo | Hoạt động trơn tru qua KWin IME |
| **GNOME (Wayland / X11)** | Wayland / X11 | 🟢 Hoàn hảo | Tích hợp Mutter Wayland Input API |
| **XFCE / i3 / bspwm / Openbox** | X11 | 🟢 Hoàn hảo | Chạy qua Fcitx5 X11 Frontend chuẩn |

---

## 💖 Lời Cảm Ơn (Credits)

Dự án **fcitx5-lilypad** xin gửi lời tri ân sâu sắc đến những tác giả và dự án mã nguồn mở đã tạo nền móng:

* **Tác giả Bamboo Engine:** Chân thành cảm ơn tác giả **Luật Nguyễn** ([BambooEngine](https://github.com/BambooEngine/bamboo-core)) — tác giả bộ engine Bamboo mã nguồn mở thuật toán xử lý âm tiết tiếng Việt tự nhiên và chuẩn xác.
* **Thư viện `bamboo-core` (Rust Native):** Chân thành cảm ơn dự án [bamboo-vi/bamboo-core](https://github.com/bamboo-vi/bamboo-core) — bản cài đặt Bamboo Core thuần Rust hiệu năng cao, zero-allocation và an toàn bộ nhớ.
* **Dự án `fcitx5-lotus`:** Chân thành cảm ơn dự án [fcitx5-lotus](https://github.com/vnlilypad/fcitx5-lotus) và tác giả **thanhpy2009** — Nguồn cảm hứng mở đường và nền móng ban đầu cho hành trình xây dựng bộ gõ tiếng Việt trên Wayland Linux.
* **Tác giả Fcitx5:** Chân thành cảm ơn **Weng Xuetian (CSSlayer)** đã tạo nên framework gõ phím Fcitx5 mạnh mẽ.
* **Tác giả `fcitx5-lilypad`:** **chiconcota** ([chiconcota](https://github.com/chiconcota)) — Thiết kế kiến trúc Sequencer, Hardware Sentinel `KEY_PAUSE`, Dynamic Adaptive Governor, và duy trì phát triển bộ gõ.

---

## 📄 Giấy Phép (License)

Dự án được phân phối dưới Giấy phép **GPL-3.0-or-later**. Xem chi tiết tại [LICENSE](../LICENSE).
