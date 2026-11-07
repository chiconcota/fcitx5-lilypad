[English](README.en.md) | [Tiếng Việt](README.md)

<a id="readme-top"></a>

<div align="center">
  <a href="https://github.com/chiconcota/fcitx5-lilypad">
    <img src="data/fcitx-lilypad-README.svg" alt="Logo" width="96" height="96">
  </a>

  <h2>🍃 Fcitx5 Lilypad</h2>

  <p>
    <b>Next-Generation Vietnamese Input Method for Linux (Wayland & X11)</b>
    <br />
    <i>Ultra-fast • Zero Key Drop • Anti-Freeze Watchdog • Auto-Adaptive Electron/Wayland Governor</i>
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
    <a href="#-installation-guide-by-distribution">
      <img src="https://img.shields.io/badge/Distros-Arch%20%7C%20Ubuntu%20%7C%20Fedora-blue.svg" alt="Distros">
    </a>
  </p>

  <p>
    <a href="#-overview"><strong>Overview</strong></a> ·
    <a href="#-key-features"><strong>Key Features</strong></a> ·
    <a href="#-installation-guide-by-distribution"><strong>Installation</strong></a> ·
    <a href="#-activation--setup"><strong>Activation</strong></a> ·
    <a href="#-credits--acknowledgments"><strong>Credits</strong></a>
  </p>
</div>

---

## 🌟 Overview

**Fcitx5 Lilypad** is a modern, high-performance Vietnamese input engine built specifically for Linux desktop environments, offering native support for both Wayland compositors and legacy X11.

Traditional Linux Vietnamese IMEs frequently suffer from notorious issues:
- ❌ **Dropped letters / swallowed keystrokes** during fast typing in web browsers and Electron applications (`thương` $\to$ `tương`, `thành` $\to$ `th`).
- ❌ **Keyboard freezing or deadlocks** when target applications lag or stall.
- ❌ **Raw digit unrolling in VNI** (`co6 thuo7g`) or double characters in Telex (`nuuo7n`) on complex web canvas apps (AFFiNE, Notion, Antigravity IDE, VS Code).
- ❌ **Inverted spaces and lost characters** near punctuation marks or right after pasting text.

👉 **Fcitx5 Lilypad was engineered to resolve all these issues once and for all!** Powered by the pure Rust [bamboo-core](https://github.com/bamboo-vi/bamboo-core) engine, combined with a **Micro-Step Sequencer** and **Dynamic Adaptive Governor**, Lilypad delivers a lightning-fast, drop-free, and natural typing experience.

---

## 🚀 Key Features

### 🦀 1. Bamboo Native Core (Pure Rust C-ABI)
- Directly integrates `bamboo-core` written in Rust — zero garbage collection, zero memory leaks, and sub-millisecond syllable composition.
- Comprehensive input methods: **Telex, VNI, VIQR, Microsoft Layout, Telex 2, Telex W, French Layout VNI**.
- Multiple charsets: **Standard Unicode, TCVN3 (ABC), VNI Windows, VIQR**.
- Intelligent **Smart Auto-Restore** for non-Vietnamese words and flexible diacritic placement (**Free Marking**).

### ⚡ 2. Sequencer Architecture & Hardware Sentinel (`KEY_PAUSE`)
- **Atomic Micro-Steps:** Clear separation of micro-deletion (Backspace) and text insertion (Commit).
- **Non-destructive Hardware Barrier (`KEY_PAUSE`):** Uses a dedicated standard hardware sentinel key as an atomic FIFO barrier, eliminating 100% of compositor race conditions without ever erasing preceding text.
- **Watchdog Hard Timeout 250ms:** Guarantees keyboard input never freezes by purging buffered context if an application hangs.

### 🧠 3. Dynamic Adaptive Governor
- **Terminal & Native Apps (Alacritty, Kitty, Foot, Kate, Gedit):** Ultra-low pre-commit settling delay of **$1.5\text{ms}$** ($1500\mu\text{s}$) and post-commit of **$0.3\text{ms}$** ($300\mu\text{s}$) for instant mechanical-keyboard responsiveness.
- **Chromium & Electron (Chrome, Brave, Edge, VS Code, Discord, Slack, AFFiNE):** Automatically synchronizes with the **1 VSync frame ($18\text{ms} \sim 22\text{ms}$)** standard (matching Lotus 20ms). This gives the Electron DOM adequate time to finish backspace deletion before committing new glyphs.

### 🛡️ 4. Virtual Focus Pinning & Focus Bounce Protection
- Automatically pins and shields word buffers when complex web apps (AFFiNE Canvas, Notion, Antigravity IDE) dispatch rapid synthetic FocusOut/FocusIn events during DOM updates.
- Native IPC over compositor sockets (`$NIRI_SOCKET`) resolves `app_id` in $155\mu\text{s}$ for sandboxed Wayland windows.

### 🎮 5. Versatile Operation Modes
- **Sequence Mode (Recommended):** Micro-step kernel uinput replacement directly inline into the text field.
- **Preedit Mode:** Traditional underline preedit candidate box with 100% compatibility.
- **Minecraft Mode:** Optimized key replacement tailored for Minecraft and fullscreen 3D games.

---

## 📦 Installation Guide by Distribution

### 1. Arch Linux / Manjaro / CachyOS / EndeavourOS

Install directly via AUR:

```bash
# Option 1: Official pre-compiled binary (Recommended):
yay -S fcitx5-lilypad-bin

# Option 2: Compile from release tarball:
yay -S fcitx5-lilypad

# Option 3: Development git branch:
yay -S fcitx5-lilypad-git
```

Or build manually from source:
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

#### Step 1: Install Build Dependencies
```bash
sudo apt update
sudo apt install -y git build-essential cmake extra-cmake-modules ninja-build \
    gettext pkg-config libx11-dev libfcitx5core-dev libfcitx5utils-dev \
    libfcitx5config-dev fcitx5-modules-dev libinput-dev libudev-dev \
    cargo rustc acl python3 python3-qtpy python3-pyqt5 fcitx5 fcitx5-config-qt
```

#### Step 2: Build & Install
```bash
git clone https://github.com/chiconcota/fcitx5-lilypad.git
cd fcitx5-lilypad/fcitx5-lilypad
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -GNinja
cmake --build build -j$(nproc)
sudo cmake --install build
```

---

### 3. Fedora / RHEL / AlmaLinux

#### Step 1: Install Build Dependencies
```bash
sudo dnf install -y git gcc-c++ make cmake extra-cmake-modules ninja-build \
    gettext pkgconf-pkg-config libX11-devel fcitx5-devel libinput-devel \
    systemd-devel cargo rust acl python3 python3-QtPy python3-qt5 \
    fcitx5 fcitx5-configtool fcitx5-autostart
```

#### Step 2: Build & Install
```bash
git clone https://github.com/chiconcota/fcitx5-lilypad.git
cd fcitx5-lilypad/fcitx5-lilypad
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -GNinja
cmake --build build -j$(nproc)
sudo cmake --install build
```

---

## ⚙️ Activation & Setup

**Sequence Mode** uses the lightweight `fcitx5-lilypad-server` background daemon to issue micro-replacement strokes via `/dev/uinput`. Enable it once:

### Step 1: Enable Uinput Permissions & Service (All Distros)
```bash
# 1. Load uinput module
sudo modprobe uinput

# 2. Initialize proxy system user (uinput_proxy)
sudo systemd-sysusers

# 3. Reload udev rules & systemd
sudo udevadm control --reload-rules && sudo udevadm trigger
sudo systemctl daemon-reload

# 4. Enable the user server daemon:
sudo systemctl enable --now fcitx5-lilypad-server@$USER.service

# 5. Restart Fcitx5:
fcitx5 -r -d
```

### Step 2: Add Lilypad to Fcitx5
1. Launch configuration GUI:
   ```bash
   fcitx5-configtool
   ```
2. Uncheck *"Only Show Current Language"*.
3. Search for **`Lilypad`** and click `>` to add it to your Input Method list.
4. Position Lilypad beneath your default English keyboard (e.g. `Keyboard - English (US)`).
5. Switch inputs using your trigger hotkey (`Ctrl + Space` or `Super + Space`) and start typing!

---

## 💖 Credits & Acknowledgments

* **Bamboo Engine:** Many thanks to **Luật Nguyễn** ([BambooEngine](https://github.com/BambooEngine/bamboo-core)) for creating the open-source Bamboo engine algorithm.
* **`bamboo-core` Rust Implementation:** Special thanks to the [bamboo-vi/bamboo-core](https://github.com/bamboo-vi/bamboo-core) team for the zero-allocation Rust native core.
* **`fcitx5-lotus` Project:** Thanks to [fcitx5-lotus](https://github.com/vnlilypad/fcitx5-lotus) and **thanhpy2009** for early architectural inspiration.
* **Fcitx5 Framework:** Kudos to **Weng Xuetian (CSSlayer)** for developing the versatile Fcitx5 framework.
* **Fcitx5 Lilypad:** **chiconcota** ([chiconcota](https://github.com/chiconcota)) — Author & Lead Architect of the Sequencer, Hardware Sentinel `KEY_PAUSE`, Dynamic Adaptive Governor, and maintainer.

---

## 📄 License

Distributed under the **GPL-3.0-or-later** License. See [LICENSE](../LICENSE) for details.
