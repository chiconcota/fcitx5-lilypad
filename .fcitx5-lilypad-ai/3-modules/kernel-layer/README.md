# MODULE: KERNEL LAYER (`fcitx5-lilypad/server/lilypad-server.cpp`)

@status: STABLE (v2.3.1-sentinel-barrier) | @last_update: 2026-08-28

---

## 1. TỔNG QUAN VÀ SỨ MỆNH (MODULE OVERVIEW)

**Kernel Layer** (`fcitx5-lilypad-server`) chịu trách nhiệm giao tiếp trực tiếp với Linux Kernel Input Subsystem qua thiết bị ảo `/dev/uinput`.

> **Nhiệm vụ cốt lõi:**
> 1. **Khởi tạo `/dev/uinput` Virtual Keyboard Device:** Đăng ký thiết bị bàn phím ảo tên `fcitx5-lilypad Virtual Keyboard` cấp kernel.
> 2. **Bắn mảng sự kiện Backspace nguyên tử (Pure Uinput Backspace Emission):** Khi Fcitx5 C++ Addon cần xóa ký tự cũ, daemon nhận số lượng phím xóa qua Unix Domain Socket (`/run/user/$UID/fcitx5-lilypad.sock`) và ghi mảng 4 sự kiện `ev[4]` (Press + SYN_REPORT + Release + SYN_REPORT) qua `/dev/uinput` trong 1 lệnh `write()` duy nhất.
> 3. **Bảo vệ cách ly người dùng (UID Socket Authentication):** Sử dụng `SO_PEERCRED` trên Unix Socket để xác thực UID của client kết nối đúng với user sở hữu daemon.
> 4. **Systemd User Service Integration:** Tự động kích hoạt dịch vụ ngầm qua Systemd Template Unit Service `fcitx5-lilypad-server@$USER.service`.

---

## 2. KIẾN TRÚC GIAO TIẾP IPC UNIX SOCKET (`lilypad-server.cpp`)

```text
 ┌────────────────────────────────────────────────────────────────────────┐
 │                 FCITX5 LILYPAD ADDON (liblilypad.so)                   │
 ├────────────────────────────────────────────────────────────────────────┤
 │ - Calculates number of Backspaces needed (deletedPart)                 │
 │ - Connects to Unix Socket: /run/user/$UID/fcitx5-lilypad.sock          │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │ (Write 4-byte backspace count)
                                     ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │            PURE KERNEL UINPUT SERVER DAEMON (lilypad-server)          │
 ├────────────────────────────────────────────────────────────────────────┤
 │ - Verifies SO_PEERCRED Unix Domain Socket authentication              │
 │ - Emits atomic ev[4] KEY_BACKSPACE events to /dev/uinput               │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │
                                     ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │                     LINUX KERNEL /dev/uinput DRIVER                    │
 └────────────────────────────────────────────────────────────────────────┘
```

---

## 3. CẤU HÌNH HỆ THỐNG & DEPLOYMENT

- **Udev Rule (`misc/99-fcitx5-lilypad-server.rules`)**:
  ```text
  KERNEL=="uinput", SUBSYSTEM=="misc", OPTIONS+="static_node=uinput", TAG+="uaccess"
  ```
- **Systemd User Unit (`misc/fcitx5-lilypad-server@.service`)**:
  ```text
  [Unit]
  Description=Fcitx5 Lilypad uinput Server Daemon for %i

  [Service]
  ExecStart=/usr/bin/fcitx5-lilypad-server -u %i
  Restart=always
  RestartSec=1s

  [Install]
  WantedBy=default.target
  ```

