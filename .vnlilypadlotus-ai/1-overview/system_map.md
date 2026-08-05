# vnlilypadlotus SYSTEM MAP & MODULE STATUS MAP (DỰ ÁN FCITX5 LILYPAD SEQUENCER)

> **Architectural Paradigm:** Hybrid Fcitx5 C++ Addon (`fcitx5-lilypad`) + Bamboo Telex Engine (Go C-FFI `bamboo-core`) + Sequencer Token Swallow Layer (`lilypad-state.cpp` + `lilypad-sequencer.cpp`).
> **Current Version:** `v2.2.0-modular-sensor` (Tái cấu trúc Modular IAckSensor Architecture, NiriAckSensor, EMA Control & Batch Replay)

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
 │  - Thư viện C-FFI (bamboo-core)  │ │  - Bắn N phím KEY_BACKSPACE qua  │
 │  - State Rebuild (EngineRebuild) │ │    /dev/uinput (1.5ms inter-gap) │
 └──────────────────────────────────┘ └──────────────────────────────────┘
```

---

## 2. BẢNG TRẠNG THÁI MODULE & THƯ MỤC NGUỒN

| Tên Module | Thư mục | Trách nhiệm chính | Trạng thái |
| :--- | :--- | :--- | :--- |
| **Lilypad C++ Addon Core** | `fcitx5-lilypad/src/` | Quản lý state context, uinput client, mode switching (Preedit, Smooth, Sequence...), và điều phối sự kiện gõ. Biên dịch ra `liblilypad.so`. | 🟢 Ready |
| **Lilypad Settings GUI** | `fcitx5-lilypad/settings-gui/` | Giao diện cấu hình PyQt (Mode Manager, App Rules, Keymap Editor). | 🟢 Ready |
| **Lotus C++ Addon Backup** | `fcitx5-lotus-main/` | Mã nguồn Lotus gốc được bảo tồn 100% làm tài liệu tham chiếu/backup. | 🟢 Reference |
| **Sequencer Token Swallow** | `src/sequencer/` | Sequencer Layer C++ (`lotus-sequencer.h/.cpp`) & Rust (`mod.rs`) hỗ trợ Serial Tagging và Wayland ACK spec. | 🟢 Ready |
| **Log Reader & Monitor** | `scripts/read_logs.sh` | Trình đọc log thời gian thực cho Fcitx5 và Lilypad. | 🟢 Ready |

---

## 3. QUY TẮC KIẾN TRÚC TOÀN CỤC (GLOBAL ARCHITECTURAL RULES)

1. **Hybrid Fcitx5 Integration Rule:** Tận dụng 100% hạ tầng Fcitx5 C++ cho Wayland/X11 IPC, Focus và UI. Tuyệt đối không viết daemon IPC độc lập gây tranh chấp tài nguyên Niri Compositor.
2. **0% EVIOCGRAB Constraint:** Không cướp phím vật lý cấp Kernel bằng `EVIOCGRAB`. Toàn bộ phím phần cứng và phím tắt (`Ctrl+C`, `Ctrl+V`, `Alt+Tab`, `Super`, `F1-F12`) chảy tự nhiên 100%.
3. **Pure Uinput Backspace Emission:** `/dev/uinput` chỉ phục vụ duy nhất mục đích bắn $N$ phím xóa `KEY_BACKSPACE` khi thực hiện thay thế ký tự thô cũ (`performReplacement`).
4. **Inter-Backspace Delay (Rule #4):** Chèn `usleep(1500)` ($1.5\,\text{ms}$) giữa các phím xóa uinput trong server daemon để Linux Kernel evdev phát từng sự kiện `SYN_REPORT` riêng biệt, tránh gộp phím.
5. **Zero deleteSurroundingText & Zero Preedit:** NGHIÊM CẤM dùng `ic_->deleteSurroundingText()` và `Preedit` trong mọi luồng gõ. 100% thao tác thay thế từ đi qua `performReplacement()` sử dụng Kernel Uinput Sequencer Layer.
6. **No-Trash Repository Standard:** Giữ repo gọn gàng, tài liệu tuân thủ nghiêm ngặt 4 ngăn kéo trong `.vnlilypadlotus-ai/`.

---

## 4. RECENT CHANGE LOG

| 2026-08-05 | Áp dụng **Proportional Backspace Micro-delay** (10-18ms) & **15ms Replay Gap** triệt hạ lỗi nuốt chữ Facebook ContentEditable DOM | `fcitx5-lilypad/src/lilypad-state.cpp` |
| 2026-08-05 | Áp dụng **Stale Serial Microstep Pruning** (`serial < active_serial_`) loại bỏ 100% vi bước cũ kẹt hàng đợi | `fcitx5-lilypad/src/lilypad-sequencer.cpp` |
| 2026-08-05 | Phân tích root cause sự kiện focus ngầm trên AFFiNE BlockSuite Canvas, tạo `affine-flags.conf` Wayland IME và rollback mã nguồn C++ về bản gốc mượt mà 100% | `~/.config/affine-flags.conf`, `fcitx5-lilypad/` |
| 2026-08-05 | Kích hoạt `calculate_adaptive_delay_ms(elapsed)` trong `receive_ack()` và nâng Safety Timeout trần lên 250ms trong `lilypad-sequencer.cpp/.h` | `lilypad-sequencer.cpp/.h` |
| 2026-08-05 | Tối ưu luồng xóa về Micro-replacement (`deletedPart`), kết hợp Caret Buffer Lock Cap & Replay Reset logic | `lilypad-state.cpp` |
| 2026-08-04 | Bổ sung hoãn nhịp vi mô `2ms` trước khi `replayBufferedKeys()` nhả phím `Space` đứng chờ trong hàng đợi để triệt hạ Race Condition trên Messenger | `lilypad-state.cpp` |
| 2026-08-04 | Kích hoạt Systemd User Service (`fcitx5-lilypad-server@chiconcota.service`) tự động khởi động cùng hệ thống và bổ sung parse UID trong `lilypad-server.cpp` | `fcitx5-lilypad/server/` |
| 2026-08-04 | Hoàn thiện tích hợp **Sequencer Layer** (Serial Tagging, Queue micro-step, Token swallow, 5ms micro-delay, 35ms ACK barrier) vào `fcitx5-lilypad` | `fcitx5-lilypad/src/` |
| 2026-08-04 | Sửa lỗi thiếu case điều hướng `LilypadMode::Sequence` trong `switch (realMode)` làm nổ rào cản Mode Sequence | `lilypad-state.cpp` |
| 2026-08-04 | Áp dụng **Backspace Passthrough Protocol** (`return false;`), cho phím xóa uinput bay xuyên vào App xóa chữ thô | `lilypad-state.cpp` |
| 2026-08-04 | Áp dụng **Universal ReplayBufferedKeys Protocol**, tái phát lại 100% phím gõ nhanh (`n` trong `thương`) trên Wayland/DBus/X11 | `lilypad-state.cpp` |
| 2026-08-04 | Thêm cờ bảo vệ **Spurious Reset Protection** chống xóa bộ đệm liên tục trong các ứng dụng Electron/Canvas Block Editor (AFFiNE) | `lilypad-state.cpp` |
| 2026-08-02 | Khởi tạo Git repo mới tinh, kết nối remote `git@github.com:chiconcota/fcitx5-lilypad.git` và push 100% code lên branch `main` | Git / GitHub |
| 2026-08-02 | Thêm chế độ gõ **`Sequence`** (UI dropdown & C++ enum mapping) vào `fcitx5-lilypad` | `fcitx5-lilypad/` |
| 2026-08-02 | Đổi tên thương hiệu độc lập **Lotus -> Lilypad** (`liblilypad.so`, `fcitx5-lilypad`), khôi phục `fcitx5-lotus-main` làm reference backup | `fcitx5-lilypad/`, `fcitx5-lotus-main/` |
| 2026-08-02 | Trích xuất và bảo tồn 3 thành phần cốt lõi (Sequencer Layer, Wayland Motion ACK spec, Log Reader) | `src/`, `scripts/` |
| 2026-08-01 | Đóng gói niêm phong v2.0.0-lotus-stable trên `main`, khởi tạo nhánh `feature/phase3-heavy-app-optimization` | Git / Docs |
| 2026-08-01 | Tích hợp Sequencer Queue Polling (`poll_next_step`) vào `handleUInputKeyPress` | `src/lotus-state.cpp` |
| 2026-08-01 | Áp dụng Sequencer Deferred Event-Loop Commit (`addTimeEvent` 2ms) triệt hạ lỗi `chaá` | `src/lotus-state.cpp`, `src/lotus-state.h` |
| 2026-08-01 | Thêm Auto-Rebuild Bamboo Engine State (`EngineRebuildFromText`) từ `oldPreBuffer_` triệt hạ lỗi `mâu4`/`đươc5` trên GTK Wayland | `src/lotus-state.cpp` |
| 2026-07-31 | Tạo `lotus-sequencer.h/.cpp` — Serial ID Tagging & Stale Token Discard | `src/lotus-sequencer.h`, `src/lotus-sequencer.cpp` |
| 2026-07-31 | Xóa sạch 100% `deleteSurroundingText` khỏi `lotus-state.cpp` | `src/lotus-state.cpp` |
| 2026-07-31 | Thêm `usleep(1500)` inter-backspace delay trong server daemon | `server/lotus-server.cpp` |
| 2026-07-31 | Bảo vệ Bamboo Core khỏi `InputContextReset` khi `is_deleting_` | `src/lotus-engine.cpp` |
| 2026-07-31 | Special Key Pass-Through (Enter/Esc/Tab) khi `is_deleting_` | `src/lotus-state.cpp` |