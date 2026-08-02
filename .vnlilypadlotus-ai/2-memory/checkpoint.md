# VNLILYPAD LOTUS CHECKPOINT (DỰ ÁN NÂNG CẤP FCITX5 LOTUS)

## 📌 Trạng thái Bàn giao Phiên Làm Việc (Session Checkpoint)

- **Tên dự án:** `vnlilypad-lotus` ("Nâng cấp Fcitx5 Lotus")
- **Đường dẫn thư mục:** `/home/chiconcota/Documents/vnlilypad-lotus/`
- **Nhánh Git làm việc:** `feature/phase3-heavy-app-optimization` (đã checkout từ tag `v2.0.0-lotus-stable`)
- **Tình trạng:** **ĐÃ NIÊM PHONG SỔ BỘ NHỚ VÀ KHÔI PHÚC HỆ THỐNG NGUYÊN BẢN HÒAN HOÀN**

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc:

1. **Trích xuất & Bảo tồn 3 Thành phần Cốt lõi:**
   - **Sequencer Layer:** Bản Rust ([src/sequencer/mod.rs](file:///home/chiconcota/Documents/vnlilypad-lotus/src/sequencer/mod.rs)) và bản C++ ([src/sequencer/lotus-sequencer.h](file:///home/chiconcota/Documents/vnlilypad-lotus/src/sequencer/lotus-sequencer.h), [src/sequencer/lotus-sequencer.cpp](file:///home/chiconcota/Documents/vnlilypad-lotus/src/sequencer/lotus-sequencer.cpp)).
   - **Wayland Motion / Frame ACK Engine:** Bản Rust ([src/wayland/mod.rs](file:///home/chiconcota/Documents/vnlilypad-lotus/src/wayland/mod.rs)) & [src/sequencer/wayland_ack_spec.md](file:///home/chiconcota/Documents/vnlilypad-lotus/src/sequencer/wayland_ack_spec.md).
   - **Trình đọc log & Monitor:** Script chạy đọc log real-time ([scripts/read_logs.sh](file:///home/chiconcota/Documents/vnlilypad-lotus/scripts/read_logs.sh)) và C++ monitor ([src/log_reader/lotus-monitor.h](file:///home/chiconcota/Documents/vnlilypad-lotus/src/log_reader/lotus-monitor.h), [src/log_reader/lotus-monitor.cpp](file:///home/chiconcota/Documents/vnlilypad-lotus/src/log_reader/lotus-monitor.cpp)).
2. **Đổi tên Thương hiệu Độc lập (Rebrand 100% Lotus -> Lilypad):**
   - Đổi tên thư mục Addon: `fcitx5-lilypad/`
   - Đổi tên toàn bộ mã nguồn C++, file cấu hình, metadata, macro `LILYPAD_`, tên hiển thị bộ gõ (**Lilypad**) và binary target (`liblilypad.so`, `fcitx5-lilypad-server`).
3. **Cài đặt Song song Hệ thống (Dual Installation: Lotus & Lilypad):**
   - **Lotus (Backup & Reference):** Đã hoàn tác và giữ nguyên 100% thư mục `fcitx5-lotus-main` làm bản tham chiếu gốc, không chỉnh sửa.
   - **Lilypad (Mã nguồn chính):** Mọi tính năng mới (bao gồm Mode `Sequence`) chỉ được phát triển và lưu trữ trên `fcitx5-lilypad/`.
   - **Fcitx5 Profile & Group:** Đã thêm cả 2 bộ gõ `lotus` và `lilypad` vào danh sách nhóm bộ gõ active của Fcitx5 qua DBus `SetInputMethodGroupInfo`.
   - **Xác nhận:** Đã kiểm tra qua DBus `CurrentInputMethod` — cả 2 bộ gõ `Lotus` và `Lilypad` đều hoạt động và chuyển đổi qua lại bình thường.
4. **Thêm Mode "Sequence" vào Giao diện Cấu hình (UI Mode Add):**
   - Đã thêm `Sequence` vào `LilypadMode` / `LotusMode` enum & i18n annotation trong C++ header (`lilypad-config.h` & `lotus-config.h`).
   - Đã thêm `MODE_SEQUENCE = 9` vào GUI Python Settings ([mode_manager.py](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/settings-gui/ui/pages/mode_manager.py) & [dynamic_settings.py](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/settings-gui/ui/pages/dynamic_settings.py)).
   - Mode `Sequence` hiện hiển thị trực quan trong menu dropdown Mode & cài đặt quy tắc ứng dụng.

---

## 📁 File Đã Thay Đổi Trong Phiên:

| File | Thay đổi |
| :--- | :--- |
| `/usr/lib/fcitx5/liblotus.so` | Thư viện Addon Lotus gốc (Đã phục hồi) |
| `/usr/lib/fcitx5/liblilypad.so` | Thư viện Addon Lilypad độc lập |
| `/usr/share/fcitx5/addon/lotus.conf` | Cấu hình Addon Lotus |
| `/usr/share/fcitx5/addon/lilypad.conf` | Cấu hình Addon Lilypad |
| `/usr/share/fcitx5/inputmethod/lotus.conf` | Cấu hình InputMethod Lotus |
| `/usr/share/fcitx5/inputmethod/lilypad.conf` | Cấu hình InputMethod Lilypad |
| `~/.config/fcitx5/profile` | Kích hoạt cả Lotus và Lilypad trong profile Fcitx5 |
| `fcitx5-lotus-main/` | Mã nguồn C++ Addon Lotus (Phục hồi 100%) |
| `fcitx5-lilypad/` | Mã nguồn C++ Addon Lilypad độc lập |

---

## 🚀 Lệnh Biên Dịch & Kiểm Thử Hệ Thống:

```bash
# Biên dịch & Cài đặt Lotus:
cd fcitx5-lotus-main/build && cmake .. && make -j$(nproc) && echo thanh123 | sudo -S make install

# Biên dịch & Cài đặt Lilypad:
cd fcitx5-lilypad/build && cmake .. && make -j$(nproc) && echo thanh123 | sudo -S make install

# Khởi động lại Fcitx5:
fcitx5 -r -d
```
