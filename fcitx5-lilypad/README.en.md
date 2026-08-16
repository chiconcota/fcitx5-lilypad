[English](README.en.md) | [Tiếng Việt](README.md)

<a id="readme-top"></a>

<div align="center">
  <a href="https://github.com/chiconcota/fcitx5-lilypad">
    <img src="data/fcitx-lilypad-README.svg" alt="Logo" width="80" height="80">
  </a>

<h2 align="center">Fcitx5 Lilypad</h2>

<p align="center">
    <b>Next-generation Vietnamese Input Method for Linux Wayland & X11</b>
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
    <a href="#installation">
      <img src="https://img.shields.io/badge/target_packaging-AUR_Arch_Linux-blue.svg" alt="AUR Target">
    </a>
    <a href="https://github.com/chiconcota/fcitx5-lilypad/issues">
      <img src="https://img.shields.io/github/issues/chiconcota/fcitx5-lilypad?style=flat&color=red" alt="Issues">
    </a>
  </p>

<p align="center">
    <a href="#installation"><strong>Installation »</strong></a>
    ·
    <a href="https://github.com/chiconcota/fcitx5-lilypad/issues/new">Report Bug</a>
    ·
    <a href="https://github.com/chiconcota/fcitx5-lilypad/issues/new">Request Feature</a>
  </p>
</div>

<br />

This project is an architectural upgrade optimized from [VMK](https://github.com/thanhpy2009/VMK). Special thanks to author Thanh for creating the baseline foundation.

---

## 🚀 Upcoming AUR Packaging Target

**`fcitx5-lilypad`** is being prepared for official packaging on the **AUR (Arch User Repository)** for Arch Linux users and derivative distros (Manjaro, EndeavourOS, Garuda Linux).

---

## 🎯 Current Status & Testing Environment

Currently, the engine is **directly optimized and tested on Niri Compositor (Arch Linux)** using the `NiriAckSensor` adaptive module.

Due to hardware constraints, the author **cannot test comprehensively across all Window Compositors and Linux Distros**. Thus, **testing and feedback from the Linux Community are highly appreciated!**

### 📊 Compatibility Matrix (Compositor & Distro)

| Window Compositor | Status | Notes | Contribution Need |
| :--- | :---: | :--- | :---: |
| **Niri** | 🟢 **Ready** | Integrated `NiriAckSensor` + EMA Control | Advanced Testing |
| **Hyprland** | 🟡 **Testing** | Universal fallback via `GenericAckSensor` | **Testers & Maintainers Needed** |
| **Sway** | 🟡 **Testing** | Supported via Wayland `zwp_input_method_v1/v2` | **Testers Needed** |
| **KDE Plasma (Wayland)**| 🟡 **Testing** | Needs IPC latency testing | **Testers Needed** |
| **GNOME (Wayland)** | 🟡 **Testing** | Needs Mutter compatibility testing | **Testers Needed** |
| **X11 (Generic)** | 🟡 **Testing** | Fallback via Fcitx5 X11 Frontend + `GenericAckSensor` (No Wayland Protocols) | **Testers Needed** |

---

## 🤝 Community Call for Testers

If you are running **Hyprland, Sway, KDE, GNOME, Fedora, Ubuntu, NixOS, Void...**, please help improve the input method by:

1. **Testing the input method** in your environment.
2. **Reporting issues** if you encounter duplicate characters or stuck buffers.
3. **Attaching Real-time Logs**: Use the script [scripts/read_logs.sh](file:///home/chiconcota/Documents/vnlilypad-lotus/scripts/read_logs.sh) when filing bug reports.
4. **Contributing Code (Pull Request)**: Implement custom ACK Sensors (`IAckSensor`) for compositors like Hyprland or Sway.

---

## 📦 Installation

### AUR (Arch Linux / Manjaro / EndeavourOS)

```bash
# Fast installation using pre-compiled binary from GitHub Releases (Recommended):
yay -S fcitx5-lilypad-bin

# Or build from latest Git main branch (Development Branch):
yay -S fcitx5-lilypad-git
```

### Build from Source

```bash
# 0. Install Fcitx5 and build dependencies
# Ubuntu / Debian:
sudo apt update && sudo apt install -y git fcitx5 fcitx5-config-qt libfcitx5core-dev libfcitx5utils-dev libfcitx5config-dev fcitx5-modules-dev libinput-dev libudev-dev extra-cmake-modules build-essential cmake gettext golang libx11-dev

# Arch Linux:
sudo pacman -S --needed git base-devel cmake extra-cmake-modules fcitx5 fcitx5-configtool fcitx5-qt fcitx5-gtk gettext fmt go libx11 libinput systemd

# Fedora:
sudo dnf install -y git gcc-c++ cmake extra-cmake-modules fcitx5-devel gettext-devel golang libX11-devel libinput-devel systemd-devel fcitx5-configtool

# 1. Clone repository
git clone https://github.com/chiconcota/fcitx5-lilypad.git
cd fcitx5-lilypad/fcitx5-lilypad

# 2. Build C++ Addon
mkdir -p build && cd build
cmake -DCMAKE_INSTALL_PREFIX=/usr ..
make -j$(nproc)
sudo make install

# 3. Check uinput module, reload systemd, user proxy & udev rules (/dev/uinput)
ls /dev/uinput || sudo modprobe uinput
sudo systemd-sysusers
sudo udevadm control --reload-rules && sudo udevadm trigger
sudo systemctl daemon-reload

# 4. Enable Server Daemon via Systemd
sudo systemctl enable --now fcitx5-lilypad-server@$USER.service

# 5. Restart Fcitx5
fcitx5 -r -d
```

---

## 📄 License

Distributed under the GPL-3.0-or-later License. See `LICENSE` for more information.
