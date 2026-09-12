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
      <img src="https://img.shields.io/badge/version-2.3.6-blue.svg" alt="Version 2.3.6">
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

## 💡 Architectural Breakthroughs (`v2.3.6 - Adaptive Dynamic Micro-Pacing & Tri-Layer Protection`)

`fcitx5-lilypad` v2.3.6 delivers a complete, robust solution to the longstanding challenges of Vietnamese IME on Linux (dropped keystrokes, character duplication, DOM desynchronization on web apps, and inverted space bars):

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
  │  - Real-time Swallow Measurement: ΔT_swallow = T2 - T1 (roundtrip)     │
  │  - Post-Commit Settling Window (70ms): Preserves RAM buffer for Webview│
  │  - Watchdog Hard Timeout (250ms) & Emergency Purge: No-Freeze Guarantee│
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │          MODULAR ACK SENSOR LAYER (ADAPTIVE LATENCY SENSORS)           │
  │  - Tri-Layer Protection: max(Finger Floor, EMA swallow, T_measured)   │
  │  - Dynamic Micro-Pacing: T_total = base_us + N * per_bs_us             │
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

### 1. Real-Time Swallow Measurement ($\Delta T_{\text{swallow}}$) & Tri-Layer Protection
- **Hardware-Level Swallow Timing:** Clocks the exact uinput roundtrip from emitting $N+1$ backspaces until Fcitx5 swallows the Sentinel token to derive raw latency $T_{\text{measured}} = \frac{\Delta T_{\text{swallow}}}{N+1}$.
- **Tri-Layer Safety Protection Formula:**
  $$\text{per\_bs\_us} = \max\Big(\underbrace{\text{min\_per\_bs\_us}}_{\text{Layer 1: Finger Floor } t}, \quad \underbrace{\text{app\_ack}_{\text{new}}}_{\text{Layer 2: Smoothed EMA}}, \quad \underbrace{T_{\text{measured}}}_{\text{Layer 3: Instant Reflex}}\Big)$$
  - **Layer 1 (Finger Floor):** Normalized Min-Max feature scaling from finger IKI speed $\in [35\text{ms}, 150\text{ms}]$. Under fast burst typing, compresses micro-delay down to **$1.0\text{ms} \sim 2.5\text{ms}$** (Zero-Latency on Terminal).
  - **Layer 2 (Smoothed EMA):** Maintains rhythm stability, filtering out sudden CPU/compositor jitter.
  - **Layer 3 (Instant Reflex):** Immediately stretches micro-delay in $1\mu\text{s}$ if the application experiences a momentary render lag or GC stall.

### 2. Uinput $N+1$ Sentinel Barrier & Post-Commit Settling Window
- **Deterministic FIFO Barrier:** Daemon emits $N+1$ backspaces; the $(N+1)$-th token is swallowed by Fcitx5 to ensure old text has been fully erased before commit.
- **Post-Commit Settling Window (70ms):** On Chromium/Electron webview applications, fast subsequent keystrokes are temporarily queued in RAM for $70\text{ms}$ while the asynchronous DOM updates, eliminating character loss in compound vowels (e.g., `"thương"` $\to$ `"tương"`).

### 3. Unified Watchdog (250ms) & Clean Architecture
- **250ms Hard Timeout:** Standardized ceiling on the Linux EventLoop triggers `purgeContextEmergency()` if an app freezes, ensuring the keyboard never hangs.
- **Zero Technical Debt:** Removed hardcoded per-app overrides, treating all Chromium/Electron applications uniformly via `ack_apps`.

---

## 💖 Acknowledgments

The **fcitx5-lilypad** project gratefully acknowledges the pioneering contributions that made this modern input method possible:

* **Bamboo Engine Author:** Special thanks to **Luật Nguyễn** ([BambooEngine](https://github.com/BambooEngine/bamboo-core)) for creating the wonderful open-source Bamboo engine — the core algorithms powering natural and accurate Vietnamese Telex/VNI syllable processing.
* **fcitx5-lilypad Author:** **Võ Ngô Hoàng Thành** ([thanhpy2009 / VMK](https://github.com/thanhpy2009)) — Chief architect behind the Sequencer, $N+1$ Sentinel Barrier, Uinput Daemon Server, IKI Adaptive Engine, and Tri-Layer Micro-Pacing.
* **Predecessor Project `fcitx5-lotus`:** Sincere thanks to [fcitx5-lotus](https://github.com/vnlilypad/fcitx5-lotus) — The pioneering initiative providing the inspiration and solid foundation for modern, smooth Vietnamese typing on Linux Wayland & X11.

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

**Step 1: Install via AUR helper (`yay` or `paru`):**
```bash
# 1. Official Pre-compiled Binary - Instant 1s install, ZERO LOGS (Recommended for end-users):
yay -S fcitx5-lilypad-bin

# 2. Official Stable Source - Builds from official release tarball, ZERO LOGS:
yay -S fcitx5-lilypad

# 3. Latest Git Master - Auto-tracks latest main branch commits, FULL LOGGING ENABLED (For Devs & Testers):
yay -S fcitx5-lilypad-git
```

**Step 2: Enable Uinput Daemon Service & Restart Fcitx5:**
```bash
# Enable and start the server daemon for your user (one-time setup):
sudo systemctl enable --now fcitx5-lilypad-server@$USER.service

# Restart Fcitx5 to apply changes:
fcitx5 -r -d
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
