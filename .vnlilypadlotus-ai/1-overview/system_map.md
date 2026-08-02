# vnlilypadlotus SYSTEM MAP & MODULE STATUS MAP (DỰ ÁN NÂNG CẤP FCITX5 LOTUS)

> **Architectural Paradigm:** Hybrid Fcitx5 C++ Addon + Bamboo Telex Engine (Go C-FFI) + Sequencer Token Swallow Layer (`lotus-state.cpp` + `lotus-sequencer.cpp`).
> **Current Version:** `v2.0.0-lotus` (trạng thái main đã khôi phục — AT-SPI2 đã gỡ)

---

## 1. TỔNG QUAN KIẾN TRÚC HYBRID (FCITX5 LOTUS UPGRADE ARCHITECTURE)

```text
 ┌────────────────────────────────────────────────────────────────────────┐
 │                   FCITX5 FRAMEWORK (HẠ TẦNG GÁC CỬA)                   │
 │  - Quản lý Wayland IPC (zwp_input_method_v2) & X11 / DBus IME Frontend │
 │  - Quản lý Focus, Window Manager, System Tray Icon & GUI Configuration │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │ (Sự kiện Phím / KeyEvent)
                                     ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │           SEQUENCER TOKEN SWALLOW LAYER (BỘ NÃO ĐIỀU PHỐI)             │
 │  - lotus-sequencer.h/.cpp: Serial ID Tagging (Serial #1, #2, #N...)    │
 │  - lotus-state.cpp: Đếm token nguyên tử expected_backspaces_           │
 │  - Cho phím xóa uinput chui vào App xóa chữ thô (return false)        │
 │  - Khi đủ N phím xóa dội về → commitString() chèn chữ có dấu 0ms      │
 └─────────────────┬──────────────────────────────────┬───────────────────┘
                   │ (Gửi phím gõ thô)                 │ (Bắn phím xóa)
                   ▼                                  ▼
 ┌──────────────────────────────────┐ ┌──────────────────────────────────┐
 │     LOTUS TELEX ENGINE (GO)      │ │   PURE UINPUT BACKSPACE EMISSION │
 │  - Xử lý quy tắc Telex/VNI       │ │  - Bắn N phím KEY_BACKSPACE qua  │
 │  - Thư viện C-FFI (bamboo-core)  │ │    /dev/uinput (1.5ms inter-gap) │
 └──────────────────────────────────┘ └──────────────────────────────────┘
```

---

## 2. BẢNG TRẠNG THÁI MODULE & THƯ MỤC NGUỒN

| Tên Module | Thư mục | Trách nhiệm chính | Trạng thái |
| :--- | :--- | :--- | :--- |
| **Fcitx5 C++ Framework** | `fcitx5-master/` | Gánh 100% Wayland/X11 IPC, Socket, Focus, Window Manager, System Tray, GUI Config. | 🟢 Ready |
| **Lotus C++ Addon Core** | `fcitx5-lotus-main/src/` | Quản lý state context, uinput client, và điều phối sự kiện gõ. Biên dịch ra `liblotus.so`. | 🟢 Ready |
| **Sequencer Token Swallow** | `fcitx5-lotus-main/src/lotus-sequencer.h/.cpp` | Serial ID Tagging, bộ đếm token `expected_backspaces_`, Sequencer Queue Polling & Event Loop Deferred Commit. | 🟢 Ready |
| **Lotus Server Daemon** | `fcitx5-lotus-main/server/lotus-server.cpp` | Phát phím xóa uinput với inter-backspace delay 1.5ms qua `/dev/uinput`. | 🟢 Ready |
| **Lotus Telex Engine** | `fcitx5-lotus-main/bamboo/` | Máy trạng thái Telex/VNI nguyên bản của lotus (Go/Bamboo C-FFI). | 🟢 Ready |
| **Sequencer Rust Reference** | `src/sequencer/` | Engine tham chiếu thuật toán Sequencer Layer (`cargo test` pass 100%). | 🟢 Ready |

---

## 3. QUY TẮC KIẾN TRÚC TOÀN CỤC (GLOBAL ARCHITECTURAL RULES)

1. **Hybrid Fcitx5 Integration Rule:** Tận dụng 100% hạ tầng Fcitx5 C++ cho Wayland/X11 IPC, Focus và UI. Tuyệt đối không viết daemon IPC độc lập gây tranh chấp tài nguyên Niri Compositor.
2. **0% EVIOCGRAB Constraint:** Không cướp phím vật lý cấp Kernel bằng `EVIOCGRAB`. Toàn bộ phím phần cứng và phím tắt (`Ctrl+C`, `Ctrl+V`, `Alt+Tab`, `Super`, `F1-F12`) chảy tự nhiên 100%.
3. **Pure Uinput Backspace Emission:** `/dev/uinput` chỉ phục vụ duy nhất mục đích bắn $N$ phím xóa `KEY_BACKSPACE` khi thực hiện thay thế ký tự thô cũ (`performReplacement`).
4. **Inter-Backspace Delay (Rule #4):** Chèn `usleep(1500)` ($1.5\,\text{ms}$) giữa các phím xóa uinput trong `lotus-server.cpp` để Linux Kernel evdev phát từng sự kiện `SYN_REPORT` riêng biệt, tránh gộp phím.
5. **Zero deleteSurroundingText & Zero Preedit:** NGHIÊM CẤM dùng `ic_->deleteSurroundingText()` và `Preedit` trong mọi luồng gõ. 100% thao tác thay thế từ đi qua `performReplacement()` sử dụng Kernel Uinput Sequencer Layer.
6. **No-Trash Repository Standard:** Giữ repo gọn gàng, tài liệu tuân thủ nghiêm ngặt 4 ngăn kéo trong `.vnlilypadlotus-ai/`.

---

## 4. RECENT CHANGE LOG

| Ngày | Thay đổi | File |
| :--- | :--- | :--- |
| 2026-08-02 | **GỠ BỎ toàn bộ AT-SPI2** — xóa `lotus-atspi.*`, gỡ khỏi CMake, sequencer quay về `WaitingForAck` (build pass 100%) | `src/` (ngoài git) |
| 2026-08-01 | Đóng gói niêm phong v2.0.0-lotus-stable trên `main`, khởi tạo nhánh `feature/phase3-heavy-app-optimization` | Git / Docs |
| 2026-08-01 | Tích hợp Sequencer Queue Polling (`poll_next_step`) vào `handleUInputKeyPress` | `src/lotus-state.cpp` |
| 2026-08-01 | Áp dụng Sequencer Deferred Event-Loop Commit (`addTimeEvent` 2ms) triệt hạ lỗi `chaá` | `src/lotus-state.cpp`, `src/lotus-state.h` |
| 2026-08-01 | Thêm Auto-Rebuild Bamboo Engine State (`EngineRebuildFromText`) từ `oldPreBuffer_` triệt hạ lỗi `mâu4`/`đươc5` trên GTK Wayland | `src/lotus-state.cpp` |
| 2026-07-31 | Tạo `lotus-sequencer.h/.cpp` — Serial ID Tagging & Stale Token Discard | `src/lotus-sequencer.h`, `src/lotus-sequencer.cpp` |
| 2026-07-31 | Xóa sạch 100% `deleteSurroundingText` khỏi `lotus-state.cpp` | `src/lotus-state.cpp` |
| 2026-07-31 | Thêm `usleep(1500)` inter-backspace delay trong server daemon | `server/lotus-server.cpp` |
| 2026-07-31 | Bảo vệ Bamboo Core khỏi `InputContextReset` khi `is_deleting_` | `src/lotus-engine.cpp` |
| 2026-07-31 | Special Key Pass-Through (Enter/Esc/Tab) khi `is_deleting_` | `src/lotus-state.cpp` |