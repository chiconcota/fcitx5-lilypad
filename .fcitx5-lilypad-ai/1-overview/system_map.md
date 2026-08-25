# fcitx5-lilypad SYSTEM MAP & MODULE STATUS MAP (DỰ ÁN FCITX5 LILYPAD SEQUENCER)

> **Architectural Paradigm:** Hybrid Fcitx5 C++ Addon (`fcitx5-lilypad`) + Bamboo Telex Engine (Go C-FFI `bamboo-core`) + Sequencer Token Swallow Layer (`lilypad-state.cpp` + `lilypad-sequencer.cpp`).
> **Current Version:** `v2.2.0-modular-sensor` (Modular IAckSensor Architecture, NiriAckSensor, EMA Machine Learning Control & Batch Replay Protocol)

---

## 1. TỔNG QUAN KIẾN TRÚC HYBRID (FCITX5 LILYPAD SEQUENCER ARCHITECTURE)

```text
  ┌────────────────────────────────────────────────────────────────────────┐
  │                   FCITX5 FRAMEWORK (HẠ TẦNG GÁC CỬA)                   │
  │  - Quản lý Wayland IPC (zwp_input_method_v2) & X11 / DBus IME Frontend │
  │  - Quản lý Focus, Window Manager, System Tray Icon & GUI Configuration │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │ (KeyEvent & InputContext)
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │            LILYPAD SEQUENCER LAYER (BỘ NÃO ĐIỀU PHỐI CHÍNH)            │
  │  - lilypad-sequencer.h/.cpp: Serial ID Tagging & MicroStep Queue       │
  │  - lilypad-state.cpp: Đếm token nguyên tử expected_swallow_backspaces_ │
  │  - Backspace Passthrough (return false): Phím xóa uinput bay tới App  │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
                                      ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │           MODULAR ACK SENSOR LAYER (CẢM BIẾN TỰ ĐỘNG THÍCH ỨNG)         │
  │  - ack-sensors/ack-sensor.h: IAckSensor Abstract Class                 │
  │  - ack-sensors/niri-sensor.h/.cpp: NiriAckSensor (EMA Machine Learning) │
  │  - ack-sensors/generic-sensor.h/.cpp: GenericAckSensor Fallback        │
  │  - ack-sensors/sensor-factory.h/.cpp: AckSensorFactory Auto-Detect     │
  └───────────────────────────────────┬────────────────────────────────────┘
                    │ (Gửi phím gõ thô)                 │ (Phát phím xóa)
                    ▼                                  ▼
  ┌──────────────────────────────────┐ ┌──────────────────────────────────┐
  │   BAMBOO TELEX ENGINE (GO C-FFI)  │ │ PURE KERNEL UINPUT SERVER DAEMON │
  │  - Engine xử lý quy tắc Telex/VNI│ │  - fcitx5-lilypad-server daemon  │
  │  - Thư viện C-FFI (bamboo-core)  │ │  - Bắn mảng 4 ev[4] KEY_BACKSPACE│
  │  - State Rebuild (EngineRebuild) │ │    qua /dev/uinput nguyên tử     │
  └──────────────────────────────────┘ └──────────────────────────────────┘
```

---

## 2. BẢNG TRẠNG THÁI MODULE & THƯ MỤC NGUỒN

| Tên Module | Thư mục | Trách nhiệm chính | Trạng thái |
| :--- | :--- | :--- | :--- |
| **Lilypad C++ Addon Core** | `fcitx5-lilypad/src/` | Quản lý state context, uinput client, mode switching (Sequence, Smooth...), và điều phối sự kiện gõ. Biên dịch ra `liblilypad.so`. | 🟢 Ready |
| **Modular ACK Sensors** | `fcitx5-lilypad/src/ack-sensors/` | Cảm biến đo thời gian `elapsed` thực tế của giao dịch, tự động điều chỉnh Dynamic Barrier thích ứng cho phím gõ tiếp theo. | 🟢 Ready |
| **Modular IKI Sensors** | `fcitx5-lilypad/src/iki-sensors/` | Cảm biến đo nhịp gõ thời gian thực ($\Delta t$), làm mịn EMA và phát hiện Burst Typing ngón tay người dùng. | 🟢 Ready |
| **Native UI & System Tray** | `fcitx5-lilypad/src/` | Dùng chung 100% giao diện Fcitx5 Lotus (`fcitx5-configtool` UI + Fcitx5 Tray Actions). | 🟢 Ready |
| **Pure Uinput Server Daemon** | `fcitx5-lilypad/server/` | Daemon phát phím xóa Backspace qua `/dev/uinput` với xác thực UID Unix Socket. | 🟢 Ready |
| **Log Reader & Monitor** | `scripts/read_logs.sh` | Trình đọc log thời gian thực cho Fcitx5 và Lilypad. | 🟢 Ready |
| **Lotus C++ Addon Backup** | `fcitx5-lotus-main/` | Mã nguồn Lotus gốc được bảo tồn 100% làm tài liệu tham chiếu/backup. | 🟢 Reference |

---

## 3. QUY TẮC KIẾN TRÚC TOÀN CỤC (GLOBAL ARCHITECTURAL RULES)

1. **Hybrid Fcitx5 Integration Rule:** Tận dụng 100% hạ tầng Fcitx5 C++ cho Wayland/X11 IPC, Focus và UI. Tuyệt đối không viết daemon IPC độc lập gây tranh chấp tài nguyên Niri Compositor.
2. **Pure Uinput Backspace Emission:** `/dev/uinput` chỉ phục vụ duy nhất mục đích bắn $N$ phím xóa `KEY_BACKSPACE` khi thực hiện thay thế ký tự thô cũ (`performReplacement`).
3. **Zero deleteSurroundingText & Zero Preedit:** NGHIÊM CẤM dùng `ic_->deleteSurroundingText()` và `Preedit` trong luồng gõ Sequence. 100% thao tác thay thế từ đi qua `performReplacement()` sử dụng Kernel Uinput Sequencer Layer.
4. **No-Trash Repository Standard:** Giữ repo gọn gàng, tài liệu tuân thủ nghiêm ngặt 4 ngăn kéo trong `.fcitx5-lilypad-ai/`. Các thử nghiệm gỡ bỏ lưu tại `2-memory/archive/deprecated-decisions.md`.

---

## 4. RECENT CHANGE LOG (LỊCH SỬ NÂNG CẤP DÒNG THỜI GIAN)

| Ngày | Mô tả nâng cấp cốt lõi | File ảnh hưởng |
| :--- | :--- | :--- |
| 2026-08-25 | Tích hợp Cold Start Safe Baseline ($>50\text{ms}$ cho chữ đầu tiên): Ấn định mức trần an toàn $50\text{ms} \sim 80\text{ms}$ cho chữ đầu tiên khi chưa có dữ liệu IKI để bảo đảm 100% không nuốt chữ, sau đó chuyển giao sang thuật toán Lerp động từ chữ thứ 2 | `fcitx5-lilypad/src/ack-sensors/`, `.fcitx5-lilypad-ai/` |
| 2026-08-25 | Hoàn thành Triển khai Phase 4.3: Tích hợp Two-Tier Timeout (Dynamic Soft Timeout theo App ACK & nhịp IKI, Watchdog Hard Timeout 250ms trên EventLoop) và cơ chế cắt lỗ khẩn cấp `purgeContextEmergency` bảo vệ tuyệt đối chuỗi phím khi App lag/freeze | `fcitx5-lilypad/src/lilypad-sequencer.h/.cpp`, `lilypad-state.h/.cpp`, `.fcitx5-lilypad-ai/` |
| 2026-08-25 | Hoàn thành Phase 4.2: Tích hợp Dynamic Micro-Pacing theo nhịp ngón tay IKI ($\alpha \in [0.15, 1.0]$) vào `IAckSensor`, `NiriAckSensor`, `GenericAckSensor` và `LilypadState`, nén trễ xuống 1.0ms~2.5ms khi Burst Typing, kiểm thử thực tế đạt chuẩn Zero-Latency siêu nhạy | `fcitx5-lilypad/src/ack-sensors/`, `lilypad-state.cpp`, `.fcitx5-lilypad-ai/` |
| 2026-08-25 | Khởi tạo nhánh `feat/iki-adaptive-engine`, thiết lập Project Manager Phase 4 (IKI Adaptive Engine), xây dựng module cảm biến độc lập `IIkiSensor` & `StandardIkiSensor` (`iki-sensors/`) đo nhịp gõ thời gian thực ($\Delta t$) qua EMA, kiểm thử log IKI thành công 100% | `fcitx5-lilypad/src/iki-sensors/`, `lilypad-state.h/.cpp`, `lilypad-config.h`, `CMakeLists.txt`, `.fcitx5-lilypad-ai/` |
| 2026-08-11 | Xây dựng hoàn chỉnh hạ tầng đóng gói AUR (`fcitx5-lilypad-git`, `fcitx5-lilypad-bin`, `fcitx5-lilypad`), tạo scriptlet `fcitx5-lilypad.install`, tự động sinh `.SRCINFO`, kiểm thử `makepkg` thành công và bổ sung hướng dẫn cài đặt AUR vào 3 README | `fcitx5-lilypad/packaging/aur/`, `README.md`, `fcitx5-lilypad/README.md`, `README.en.md` |
| 2026-08-09 | Nhúng 100% mã nguồn Go `bamboo-core` vào Git main, sửa `Library=liblilypad` & `[Dependencies]` trong addon config, sửa đường dẫn systemd service `/lib/systemd/system`, tối ưu nhịp replay `3000:300` và lập trình cờ Release Zero-Log Overhead | `fcitx5-lilypad/bamboo/bamboo-core/`, `lilypad-addon.conf.in.in`, `misc/CMakeLists.txt`, `lilypad-state.cpp`, `lilypad-utils.h` |
| 2026-08-08 | Chuẩn hóa toàn bộ hệ thống tài liệu sang `fcitx5-lilypad` & `.fcitx5-lilypad-ai/`, bổ sung bộ `.agent/README.md` cho AI Kit, dọn dẹp thư mục rác `src/` (code Rust cũ) và `fcitx5-lotus-main.zip` | `.fcitx5-lilypad-ai/`, `.agent/`, `README.md` |
| 2026-08-07 | Chuẩn hóa hướng dẫn cài đặt trong 3 README (`ls /dev/uinput`, `systemd-sysusers`, `udevadm reload`, `fcitx5 -r -d`) và sửa dứt điểm lỗi mất tiếng Việt sau Logout/Login (`DefaultIM=lilypad`) | `README.md`, `fcitx5-lilypad/README.md`, `README.en.md`, `~/.config/fcitx5/profile` |
| 2026-08-06 | Tối ưu hóa toàn bộ hệ thống tài liệu (`1-overview`, `2-memory`, `3-modules`). Lưu trữ các thử nghiệm gỡ bỏ vào `archive/deprecated-decisions.md` | `.fcitx5-lilypad-ai/` |
| 2026-08-06 | Viết tài liệu đặc tả kiến trúc kỹ thuật chi tiết **NiriAckSensor & Modular IAckSensor Architecture** kèm công thức EMA toán học, luồng 5 bước Linux Wayland Input Subsystem và Bằng chứng thực nghiệm log Messenger | `.fcitx5-lilypad-ai/3-modules/sequencer-layer/niri-ack-sensor-architecture.md` |
| 2026-08-06 | Chuẩn hóa trang GitHub (README.md, README.en.md), bổ sung Ma trận Tương thích Compositor (Niri 🟢, Hyprland/Sway/KDE/GNOME 🟡) và Kêu gọi Cộng đồng Đóng góp (Call for Testers) | `README.md`, `fcitx5-lilypad/README.md`, `README.en.md` |
| 2026-08-05 | Áp dụng **Optimized Batch Replay Protocol** (0.1ms cho phím chữ, 3ms cho Space) triệt hạ hoàn toàn lỗi kẹt phím 4.5s khi gõ siêu tốc | `fcitx5-lilypad/src/lilypad-state.cpp` |
| 2026-08-05 | Áp dụng **Modular IAckSensor Architecture** (`IAckSensor`, `NiriAckSensor`, `GenericAckSensor`, `AckSensorFactory`) tự động nạp theo môi trường `$XDG_CURRENT_DESKTOP` | `fcitx5-lilypad/src/ack-sensors/` |
| 2026-08-05 | Kích hoạt `calculate_adaptive_delay_ms(elapsed)` trong `receive_ack()` và nâng Safety Timeout trần lên 250ms trong `lilypad-sequencer.cpp/.h` | `lilypad-sequencer.cpp/.h` |
| 2026-08-05 | Áp dụng **Stale Serial Microstep Pruning** (`serial < active_serial_`) loại bỏ 100% vi bước cũ kẹt hàng đợi | `fcitx5-lilypad/src/lilypad-sequencer.cpp` |
| 2026-08-04 | Kích hoạt Systemd User Service (`fcitx5-lilypad-server@.service`) tự động khởi động cùng hệ thống và bổ sung parse UID trong `lilypad-server.cpp` | `fcitx5-lilypad/server/` |
| 2026-08-04 | Hoàn thiện tích hợp **Sequencer Layer** (Serial Tagging, Queue micro-step, Token swallow, 5ms micro-delay, 35ms ACK barrier) vào `fcitx5-lilypad` | `fcitx5-lilypad/src/` |
| 2026-08-02 | Khởi tạo Git repo mới tinh, kết nối remote `git@github.com:chiconcota/fcitx5-lilypad.git` và push 100% code lên branch `main` | Git / GitHub |