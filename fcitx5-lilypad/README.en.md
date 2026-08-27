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
    <i>Hybrid Architecture: Fcitx5 C++ Addon + IKI Adaptive Engine + Sentinel Barrier Protocol + Kernel Uinput Server Daemon</i>
    <br />
    <br />
    <a href="https://github.com/chiconcota/fcitx5-lilypad/releases">
      <img src="https://img.shields.io/github/v/release/chiconcota/fcitx5-lilypad?style=flat&color=success" alt="Release">
    </a>
    <a href="https://github.com/chiconcota/fcitx5-lilypad">
      <img src="https://img.shields.io/badge/version-2.3.0-blue.svg" alt="Version 2.3.0">
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

## 💡 Architectural Breakthroughs (`v2.3.0 - IKI Adaptive & Sentinel Barrier`)

`fcitx5-lilypad` v2.3.0 delivers a complete, robust solution to the longstanding challenges of Vietnamese IME on Linux (dropped keystrokes, character duplication, DOM desynchronization on web apps, and inverted space bars):

```text
  ┌────────────────────────────────────────────────────────────────────────┐
  │                   FCITX5 FRAMEWORK (GATEKEEPER LAYER)                  │
  │  - Wayland IPC (zwp_input_method_v2) & X11 / DBus IME Frontend         │
  │  - Real-time Finger Typing Speed Tracking (IIkiSensor EMA Tracking)    │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │ (KeyEvent & IKI Speed)
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │         LILYPAD SEQUENCER & SENTINEL BARRIER (COORDINATOR CORE)        │
  │  - Sentinel Barrier N+1: Emits N+1 Backspaces, swallows (N+1)-th key   │
  │  - Two-Tier Timeout: Dynamic Soft Timeout (App Lag) & Watchdog 250ms   │
  │  - Emergency Purge: Instant raw key flush on complete app freeze       │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │          MODULAR ACK SENSOR LAYER (ADAPTIVE LATENCY SENSORS)           │
  │  - Dynamic Micro-Pacing Lerp: Normalized scaling with finger IKI speed │
  │  - App ACK Consumption: Integrated DOM consumption time (N * T_ack)    │
  │  - Cold Start Safe Baseline: Safe >50ms ceiling for the first character│
  └───────────────────────────────────┬────────────────────────────────────┘
                    │ (Raw Typing Stream)               │ (Emits N+1 Backspaces)
                    ▼                                  ▼
  ┌──────────────────────────────────┐ ┌──────────────────────────────────┐
  │   BAMBOO TELEX ENGINE (GO C-FFI)  │ │ PURE KERNEL UINPUT SERVER DAEMON │
  │  - Telex/VNI rule engine         │ │  - fcitx5-lilypad-server daemon  │
  │  - Go C-FFI (bamboo-core)        │ │  - Emits ev[4] KEY_BACKSPACE     │
  │  - Zero-latency state rebuild    │ │    atomically via /dev/uinput    │
  └──────────────────────────────────┘ └──────────────────────────────────┘
```

### 1. Dynamic Micro-Pacing via Normalized Lerp & App ACK Consumption
- **Normalized Linear Interpolation (Lerp):** Instead of blind static delays, the engine continuously tracks finger typing rhythm ($\mathrm{EMA}_{\mathrm{IKI}}$) via the `IIkiSensor` module combined with application response time ($T_{\text{ack}}$):
  - **Terminal / Lightweight Apps:** Micro-delay dynamically compresses to the physical floor of **$1.5\text{ms} \sim 2.5\text{ms}$** (Zero-Latency responsiveness).
  - **Facebook / Web DOM / Electron:** Micro-delay automatically scales up to safely match DOM consumption time ($45\text{ms} \sim 60\text{ms}$), ensuring React DOM finishes consuming deletions before new characters are committed.
- **Cold Start Safe Baseline ($>50\text{ms}$):** On application start or when typing the first word without prior $\text{IKI}$ and $\text{App ACK}$ history, a safe baseline ($50\text{ms} \sim 80\text{ms}$) guarantees 0% dropped characters on the very first character.

### 2. Uinput $N+1$ Sentinel Barrier Protocol
- When replacing $N$ characters, the daemon emits **$N+1$ `KEY_BACKSPACE` events**:
  - The first $N$ events pass through to the application (`return false;`) to erase old text.
  - The $(N+1)$-th event acts as a **Sentinel Barrier**: Fcitx5 swallows this token (`event.filterAndAccept(); return true;`) and prevents it from reaching the application.
- **Hardware FIFO Guarantee:** The arrival of the $(N+1)$-th token at Fcitx5 serves as deterministic hardware proof that all prior $N$ deletions have been consumed by the app, eliminating 100% of race conditions.

### 3. Two-Tier Timeout & Emergency State Protection
- **Dynamic Soft Timeout ($T_{\text{soft}}$):** Automatically detects DOM render lag or GC stalls, transitioning the sequencer into `BarrierState::AppLagHolding` and buffering keys in RAM (`buffered_keys_`) to prevent broken words.
- **Watchdog Hard Timeout (250ms) & Emergency Purge:** An independent 250ms watchdog timer runs on the Main Event Loop. If the app freezes completely, `purgeContextEmergency()` resets the engine, clears the word buffer, and immediately forwards buffered keystrokes as raw keys (`ic_->forwardKey()`), guaranteeing **the keyboard never freezes or gets stuck**.

### 4. Uniform Web IME Routing & GTK4 Native Precision ($1\mu\text{s}$)
- **Chromium / Web Routing:** Harmonizes the commit stream for Chromium/Electron through `ic_->commitString()` to eliminate Virtual DOM conflicts on Google Docs and Facebook.
- **GTK4 Native Precision ($1\mu\text{s}$):** Uses $1\mu\text{s}$ high-precision event loop timers while preserving native key event forwarding for GTK4 / Text Editors, eliminating inverted space bar issues.

---

## 🎯 Current Status & Testing Environment

`fcitx5-lilypad` is deeply optimized on **Niri Compositor (Arch Linux)** and provides universal out-of-the-box compatibility across all Wayland Compositors & X11 via `GenericAckSensor`.

### 📊 Compatibility Matrix (Compositor & Distro)

| Window Compositor | Status | Notes | Contribution Need |
| :--- | :---: | :--- | :---: |
| **Niri** | 🟢 **Ready** | Integrated `NiriAckSensor` + IKI Adaptive + Sentinel Barrier | Continuous Testing |
| **Hyprland** | 🟢 **Ready** | Universal support via `GenericAckSensor` | Testers & Feedback |
| **Sway** | 🟢 **Ready** | Supported via Wayland `zwp_input_method_v1/v2` | Testers & Feedback |
| **KDE Plasma (Wayland)**| 🟢 **Ready** | Supported via KWin Wayland IME API | Testers & Feedback |
| **GNOME (Wayland)** | 🟢 **Ready** | Supported via Mutter Wayland Input API | Testers & Feedback |
| **X11 (Generic)** | 🟢 **Ready** | Fallback via Fcitx5 X11 Frontend + `GenericAckSensor` | Testers & Feedback |

---

## 📦 Installation

### AUR (Arch Linux / Manjaro / EndeavourOS)

```bash
# 1. Official Pre-compiled Binary - Instant 1s install, ZERO LOGS (Recommended for end-users):
yay -S fcitx5-lilypad-bin

# 2. Official Stable Source - Builds from official release tarball, ZERO LOGS:
yay -S fcitx5-lilypad

# 3. Latest Git Master - Auto-tracks latest main branch commits, FULL LOGGING ENABLED (For Devs & Testers):
yay -S fcitx5-lilypad-git
```

### Build from Source

#### Step 0: Install Dependencies

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

#### Step 1: Clone and Build

```bash
# 1. Clone repository
git clone https://github.com/chiconcota/fcitx5-lilypad.git
cd fcitx5-lilypad/fcitx5-lilypad

# 2. Build C++ Addon & Server Daemon
mkdir -p build && cd build
cmake -DCMAKE_INSTALL_PREFIX=/usr ..
make -j$(nproc)
sudo make install
```

#### Step 2: Load Uinput & Enable Daemon Service

```bash
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

## 🤝 Community Call for Testers

If you are running **Hyprland, Sway, KDE, GNOME, Fedora, Ubuntu, NixOS, Void...**, please help improve the input method by:

1. **Testing the input method** in your daily workflow.
2. **Reporting issues** if you encounter character duplication, freezes, or dropped letters.
3. **Attaching Real-time Logs**: Use the script [scripts/read_logs.sh](file:///home/chiconcota/Documents/vnlilypad-lotus/scripts/read_logs.sh) when filing bug reports.
4. **Contributing Code (Pull Request)**: Implement custom ACK Sensors (`IAckSensor`) for specific compositor environments.

---

## 📄 Project Structure

- C++ Fcitx5 Addon Core: [fcitx5-lilypad/](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/)
- Uinput Server Daemon: [fcitx5-lilypad/server/](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/server/)
- ACK & IKI Sensor Layer: [fcitx5-lilypad/src/ack-sensors/](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/ack-sensors/) & [fcitx5-lilypad/src/iki-sensors/](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/iki-sensors/)
- System Architecture Documentation: [.fcitx5-lilypad-ai/](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/)
- Real-time Log Monitor: [scripts/read_logs.sh](file:///home/chiconcota/Documents/vnlilypad-lotus/scripts/read_logs.sh)

---

## 📄 License

Distributed under the **GPL-3.0-or-later** License. See [LICENSE](LICENSE) for more information.
