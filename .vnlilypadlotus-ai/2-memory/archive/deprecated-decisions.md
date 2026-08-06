# ARCHIVE: CÁC QUYẾT ĐỊNH & THỬ NGHIỆM ĐÃ HỦY BỎ [XOÁ - KHÔNG ÁP DỤNG]

Tài liệu này lưu trữ toàn bộ các phương án kỹ thuật, quyết định kiến trúc và thử nghiệm đã từng được nghiên cứu nhưng **ĐÃ GỠ BỎ HOÀN TOÀN / KHÔNG ÁP DỤNG** trong mã nguồn thực tế của dự án `fcitx5-lilypad` (`vnlilypad-lotus`).

---

## ❌ 1. [XOÁ - KHÔNG ÁP DỤNG] AT-SPI2 DBus DOM ACK Engine (Gỡ bỏ ngày 2026-08-02)
- **Mô tả:** Lắng nghe tín hiệu `org.a11y.atspi.Event.Text` từ Linux Accessibility Bus qua DBus `/run/user/$UID/at-spi/bus_1` để phát hiện khi Web App (Chrome/Firefox) cập nhật DOM.
- **Lý do gỡ bỏ:**
  - Hầu hết các ứng dụng Chrome/Firefox/Electron trên Linux mặc định KHÔNG bật cờ Accessibility (`GTK_MODULES`/`ACCESSIBILITY_ENABLED` trống) $\rightarrow$ Bus `/at-spi/bus_1` không tồn tại, app không phát signal.
  - Gây tốn tài nguyên DBus IPC vô ích và làm phức tạp hóa mã nguồn C++.
- **Quyết định thay thế:** Gỡ bỏ 100% mã nguồn `lotus-atspi.h/.cpp` (Quyết định 040). Thay thế bằng **Modular IAckSensor Architecture** (`NiriAckSensor` + EMA Adaptive Control & `GenericAckSensor`).

---

## ❌ 2. [XOÁ - KHÔNG ÁP DỤNG] Cướp Phím Phần Cứng Cấp Kernel (`EVIOCGRAB`) (Gỡ bỏ ngày 2026-07-28)
- **Mô tả:** Sử dụng lệnh ioctl `EVIOCGRAB` để chiếm quyền độc quyền thiết bị bàn phím phần ứng `/dev/input/event*`.
- **Lý do gỡ bỏ:**
  - Gây nguy cơ đóng băng bàn phím hệ thống (Keyboard Freeze Panic) nếu tiến trình bị crash hoặc panic.
  - Phá vỡ các phím tắt hệ thống (`Ctrl+C`, `Ctrl+V`, `Alt+Tab`, `Super+Space`) và các phím chức năng (`F1-F12`, Mũi tên, Volume).
- **Quyết định thay thế:** Áp dụng **0% EVIOCGRAB Constraint** (Quyết định 008). Để phím phần ứng chảy tự nhiên 100% qua Fcitx5 Framework, chỉ dùng `/dev/uinput` cho mục đích duy nhất là bắn phím xóa `KEY_BACKSPACE`.

---

## ❌ 3. [XOÁ - KHÔNG ÁP DỤNG] Giao Thức `deleteSurroundingText()` (Gỡ bỏ ngày 2026-07-31)
- **Mô tả:** Gọi trực tiếp `ic_->deleteSurroundingText()` của Fcitx5 IPC API để xóa ký tự xung quanh con trỏ.
- **Lý do gỡ bỏ:**
  - Gây xung đột dữ liệu giữa Fcitx5 IPC và Kernel Uinput Sequencer Layer.
  - Làm nhảy con trỏ, đứt đoạn Telex trên các ứng dụng Electron và Web Editors (AFFiNE, ONLYOFFICE, Messenger).
- **Quyết định thay thế:** Nghiêm cấm 100% `deleteSurroundingText()`. 100% thao tác xóa đi qua `performReplacement()` bằng **Pure Kernel Uinput Server Daemon**.

---

## ❌ 4. [XOÁ - KHÔNG ÁP DỤNG] `std::this_thread::sleep_for()` / `usleep()` Trên Main Event Loop Thread (Gỡ bỏ ngày 2026-07-31)
- **Mô tả:** Sử dụng lệnh sleep cứng trên thread chính của Fcitx5 để tạo độ trễ giữa các phím xóa.
- **Lý do gỡ bỏ:**
  - Đóng băng toàn bộ Fcitx5 Main Event Loop, làm liệt phím bàn phím người dùng trong thời gian sleep.
- **Quyết định thay thế:** Nghiêm cấm sleep trên Main Thread. Sử dụng **Fcitx5 Async EventLoop Timer (`addTimeEvent`)** không khóa để hoãn nhịp.

---

## ❌ 5. [XOÁ - KHÔNG ÁP DỤNG] Standalone Rust `WaylandImeClient` Prototype (Gỡ bỏ ngày 2026-08-02)
- **Mô tả:** Tiến trình Rust độc lập (`src/wayland/`, `wayland-client v0.31`) cố gắng kết nối trực tiếp `$WAYLAND_DISPLAY` và sử dụng `delete_surrounding_text`.
- **Lý do gỡ bỏ:**
  - Tranh chấp tài nguyên socket với Fcitx5 và Niri Compositor, gây ra lỗi đè phím và rác chữ.
- **Quyết định thay thế:** Chuyển sang **Addon C++ Thuần túy tích hợp Fcitx5 (`fcitx5-lilypad`)**, tận dụng 100% hạ tầng IPC của Fcitx5 và kết nối Uinput Server Daemon (`fcitx5-lilypad-server`).

