# KERNEL LAYER (v2.0 Architecture Specification)

@module: `kernel-layer` | @path: `src/kernel/` | @target: `/dev/input/event*` + `/dev/uinput` | @status: 🟢 Done (v2.0 Hybrid)

---

## 1. TỔNG QUAN VÀ SỨ MỆNH (MODULE OVERVIEW)

`Kernel Layer` là tầng giao tiếp phần cứng cấp thấp của `vnlilypad`, chịu trách nhiệm:
1. **Đánh chặn phím phần cứng (Evdev Interception):** Mở thiết bị bàn phím vật lý qua `/dev/input/event*` và áp dụng `EVIOCGRAB` để chiếm quyền điều khiển phím thô (`src/kernel/evdev.rs`).
2. **Khai tử Bàn phím ảo Wayland (`zwp_virtual_keyboard_v1`):** Thay thế hoàn toàn giao thức bàn phím ảo của Wayland (vốn bị lỗi nháy `DEACTIVATE` trên GTK4 và nuốt phím Backspace trên Electron).
3. **Giả lập Bàn phím Cứng ảo (`/dev/uinput` Driver):** Khởi tạo một thiết bị bàn phím ảo chuẩn Kernel (`src/kernel/uinput.rs`) để phát phím thô (`KEY_BACKSPACE`, `KEY_ENTER`, `KEY_A`...) cho hệ thống khi Sequencer Layer yêu cầu.
4. **Panic Safety & No-Freeze Watchdog:** Tích hợp bộ giải cứu khẩn cấp signal handler (`SIGINT`, `SIGTERM`) tự động nhả `EVIOCGRAB` tức thì nếu daemon gặp sự cố, đảm bảo không bao giờ đơ bàn phím toàn hệ thống.

---

## 2. KIẾN TRÚC PHÂN CÁCH TRÁCH NHIỆM (DECOUPLED ARCHITECTURE)

```text
 ┌────────────────────────────────────────────────────────────────────────┐
 │                         Sequencer Layer                                │
 │       (Điều phối nhả phím / Micro-Step Async State Machine)            │
 └───────────────────┬────────────────────────────────┬───────────────────┘
                     │                                │
                     ▼                                ▼
 ┌───────────────────────────────────────┐ ┌──────────────────────────────┐
 │             Kernel Layer              │ │        Wayland Layer         │
 │     (/dev/input + /dev/uinput)        │ │    (zwp_input_method_v2)     │
 ├───────────────────────────────────────┤ ├──────────────────────────────┤
 │ - Captures raw hardware keys (evdev)  │ │ - Receives App Focus Sync    │
 │ - Emits physical hardware key events  │ │ - Receives SurroundingText   │
 │ - Eliminates Wayland virtual keyboard │ │ - Sends CommitString &       │
 │ - Emergency Panic Grab Recovery       │ │   SetPreeditString to App    │
 └───────────────────────────────────────┘ └──────────────────────────────┘
```

---

## 3. GIAO TIẾP VÀ TÍCH HỢP HỆ THỐNG

- **`src/kernel/evdev.rs`**: Tự động lọc qua `EVIOCGNAME` để tránh grab nhầm thiết bị uinput ảo của chính daemon, quản lý vòng đời grab/ungrab phím an toàn.
- **`src/kernel/uinput.rs`**: Tạo thiết bị `/dev/uinput` tên `vnlilypad Virtual Hardware Keyboard`, xử lý phát sự kiện phím thô `emit_key` và `emit_click` với nhịp trễ vi mô $1\,\text{ms}$.
- **Giao tiếp với Sequencer & Wayland Layer:** `MicroStep::ForwardKey` từ Sequencer được đẩy trực tiếp xuống `/dev/uinput` driver, `CommitString` và `SetPreeditString` đẩy qua Wayland IPC socket.
