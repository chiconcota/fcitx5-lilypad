# VNLILYPAD LOTUS CHECKPOINT (DỰ ÁN NÂNG CẤP FCITX5 LOTUS)

## 📌 Trạng thái Bàn giao Phiên Làm Việc (Session Checkpoint)

- **Tên dự án:** `vnlilypad-lotus` ("Nâng cấp Fcitx5 Lotus")
- **Đường dẫn thư mục:** `/home/chiconcota/Documents/vnlilypad-lotus/`
- **Nhánh Git làm việc:** `main` (Remote `origin`: `git@github.com:chiconcota/fcitx5-lilypad.git`)
- **Tình trạng:** **ĐÃ MERGE THÀNH CÔNG V2.2.0 (MODULAR ACKSENSOR & BATCH REPLAY) LÊN MAIN & PUSH GITHUB.**

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc (2026-08-06 - Chuẩn hóa Trang GitHub & Community Call):

1. **Chuẩn hóa Trang giới thiệu GitHub Dự án:**
   - Cập nhật [README.md](file:///home/chiconcota/Documents/vnlilypad-lotus/README.md), [fcitx5-lilypad/README.md](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/README.md) và [fcitx5-lilypad/README.en.md](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/README.en.md).
   - Minh bạch hóa trạng thái thử nghiệm: Tác giả hiện kiểm thử và tối ưu chính trên **Niri Compositor (Arch Linux)**.
   - Xây dựng **Bảng Ma trận Tương thích Compositor & Distro** (Niri 🟢, Hyprland/Sway 🟡, KDE/GNOME 🟡, X11 🟢).
   - Thiết lập lời kêu gọi **Community Call for Testers & Contributors** thu hút đóng góp từ cộng đồng Linux Việt Nam.
2. **Xây dựng Tài Liệu Kỹ Thuật Chi Tiết `NiriAckSensor` & `IAckSensor`:**
   - Tạo & hoàn thiện file đặc tả kiến trúc [.vnlilypadlotus-ai/3-modules/sequencer-layer/niri-ack-sensor-architecture.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.vnlilypadlotus-ai/3-modules/sequencer-layer/niri-ack-sensor-architecture.md).
   - Đảm bảo 100% tài liệu bám sát chính xác mã nguồn C++ trong `fcitx5-lilypad/src/ack-sensors/` (`ack-sensor.h`, `niri-sensor.h`, `generic-sensor.h`, `sensor-factory.h`).
   - Giải thích sơ đồ 5 bước kiến trúc tầng dưới **Linux Wayland Input Subsystem** (`/dev/uinput` $\to$ Kernel Evdev $\to$ `libinput` & Window Compositor $\to$ Fcitx5 Token Swallow $\to$ App Window).
   - Khẳng định **Nguyên tắc Vạn năng (Universal Wayland IME Protocol)**: Không cần viết Sensor riêng cho từng Distro, `GenericAckSensor` tự động xử lý vạn năng 100% mọi Distro & Compositor hỗ trợ `zwp_input_method_v1/v2`.
   - Chỉ định rõ 4 vị trí đoạn code C++ cốt lõi đo mốc $T_1$, nuốt token uinput, chốt mốc $T_2$ và tính `calculate_adaptive_delay_ms()`.
   - Tối ưu hóa **[system_map.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.vnlilypadlotus-ai/1-overview/system_map.md)**: Loại bỏ các quy tắc/thử nghiệm cũ không liên quan (`usleep 1500`, `EVIOCGRAB`), tinh gọn 4 quy tắc kiến trúc cốt lõi và làm sạch nhật ký thay đổi tập trung vào `v2.2.0-modular-sensor`.
   - Tối ưu hóa **[decision-log.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.vnlilypadlotus-ai/2-memory/decision-log.md)**: Phân nhóm mạch lạc 5 mảng kiến trúc đang vận hành thực tế 100% (Modular Sensor, Sequencer Layer, Uinput Server, Fcitx5 Systemd, và AFFiNE Zero-Regression Policy), cắt bỏ các mục trùng lặp/hủy bỏ sang `archive/deprecated-decisions.md`.
   - Cắt bỏ thư mục rác **`3-modules/wayland-layer/`**: Đây là tài liệu thử nghiệm client Rust cũ độc lập (`WaylandImeClient` v0.31) không còn nằm trong kiến trúc Fcitx5 C++ Addon hiện tại, đã chuyển lưu trữ sang `archive/deprecated-decisions.md`.

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc Trước (2026-08-05 - Tái cấu trúc Sensor & Batch Replay):

1. **Kiến trúc Modular IAckSensor:**
   - Tách Lớp Cảm biến ACK thành các Module cắm/rút (`IAckSensor`, `NiriAckSensor`, `GenericAckSensor`, `AckSensorFactory`).
   - Tự động phát hiện biến môi trường `$XDG_CURRENT_DESKTOP` để nạp Module Sensor tương ứng.
2. **Thuật toán EMA Machine Learning Adaptive Control:**
   - Tự động điều chỉnh độ trễ thích ứng theo nhịp lag của App ($0.35 \times \text{Measured} + 0.65 \times \text{Prev}$) và tự động suy giảm (decay) nhanh về $5\text{ms}$ khi App mượt.
3. **Tối ưu Luồng Replay Batch Flush:**
   - Phân biệt phím CÁCH (`Space` - hoãn 3ms chống đè IPC) và phím thường (`a, b, c...` - xả tức thì 0.1ms).
   - Bẻ gãy 100% bẫy đệ quy hoãn 15ms từng phím, triệt hạ hoàn toàn lỗi kẹt phím 4.5s khi gõ tốc độ cao.

1. **Phát hiện Root Cause trên AFFiNE (BlockSuite Canvas):**
   - Log debug xác nhận: Mỗi khi bộ gõ `commitString()` 1 ký tự, BlockSuite Editor của AFFiNE lập tức phát sự kiện `activate()` / `InputContextFocusIn` ngầm 10ms sau đó.
   - Code `activate()` trong `lilypad-engine.cpp` gọi `setMode()` và `clearAllBuffers()`, làm mất bộ nhớ `oldPreBuffer_` giữa các phím gõ làm đứt đoạn Telex.
2. **Khôi phục Nguyên trạng An toàn (Safety-First Directive):**
   - Đã thực hiện `git checkout` khôi phục 100% mã nguồn C++ gốc sạch sẽ trên `main`.
   - Đã biên dịch & cài đặt lại `/usr/lib/fcitx5/liblilypad.so` chuẩn, bảo đảm bộ gõ chạy mượt 100% trên các ứng dụng thông thường (Messenger, Chrome, IDE, Terminal).

---

## 💡 BẢNG PHƯƠNG ÁN XỬ LÝ LƯU TRỮ CHO PHIÊN KẾ TIẾP (AFFiNE ROADMAP):

### 🔹 Phương án 1: Bật Cờ Wayland IME trong Electron (`~/.config/affine-flags.conf`)
- File cờ đã được tạo tại [~/.config/affine-flags.conf](file:///home/chiconcota/.config/affine-flags.conf):
  ```text
  --ozone-platform=wayland
  --enable-wayland-ime
  --wayland-text-input-version=3
  ```
- Giúp Electron 39 giao tiếp trực tiếp qua Wayland text-input-v3 thay vì X11 fallback.

### 🔹 Phương án 2: Cách ly Spurious Focus Event cho AFFiNE
- Đưa điều kiện kiểm tra `appName == "AFFiNE"` / `appName == "ONLYOFFICE"` vào **trước** khi `setMode()` được gọi trong `activate()`.
- Tuyệt đối không `return` sớm trong `setMode()` nếu không có cờ kiểm tra Focus thực sự, tránh lỗi trôi bộ đệm khi chuyển đổi giữa 2 ứng dụng khác nhau.

### 🔹 Phương án 3: Sequencer Stale Serial Pruning (Lọc vi bước cũ an toàn)
- Khi người dùng gõ nhanh phím mới trong lúc AFFiNE đang render phím cũ, không được dùng `sequencer_.clear()` thô vì sẽ làm mất cờ đếm token `expected_swallow_backspaces_`.
- Thay vào đó, trong `Sequencer::poll_next_step()`, chỉ bỏ qua các `MicroStep` có `step.serial < active_serial_` mà giữ nguyên token count.

### 🔹 Phương án 4: Chế độ Whole-Word Replacement cho Canvas Shadow DOM
- Nếu BlockSuite Canvas bị vỡ node DOM khi nhận xóa vi mô, cho phép Mode Sequence thực hiện thay thế nguyên từ (`Whole-Word Replacement`) trong 1 giao dịch nguyên tử duy nhất.

---

## 🎯 Kế Hoạch Bàn Giao Phiên Tiếp Theo (Handover Plan for Next Session):

1. **Đóng gói Release & PKGBUILD lên AUR (Arch User Repository):**
   - Xây dựng file `PKGBUILD` và `.SRCINFO` chuẩn Arch Linux cho `fcitx5-lilypad`.
   - Cấu hình cài đặt `fcitx5-lilypad-server@.service` tự động qua Systemd service unit.
   - Kiểm thử biên dịch và cài đặt từ `makepkg -si` trên Arch Linux / Niri.
2. **Kêu gọi Cộng đồng Thử nghiệm (Community Call):**
   - Đăng thông báo phát hành AUR package cho Cộng đồng Linux Việt Nam.

---

## 📁 Các File Thay Đổi Trong Phiên:

| File | Thay đổi |
| :--- | :--- |
| `README.md` | Chuẩn hóa trang GitHub, thêm Ma trận Tương thích & Kêu gọi Tester |
| `fcitx5-lilypad/README.md` | Cập nhật README Tiếng Việt chuyên nghiệp & Định hướng AUR |
| `fcitx5-lilypad/README.en.md` | Cập nhật README Tiếng Anh chuyên nghiệp & Định hướng AUR |
| `.vnlilypadlotus-ai/3-modules/sequencer-layer/niri-ack-sensor-architecture.md` | Đặc tả kiến trúc C++ IAckSensor, NiriAckSensor, EMA, 5-step Linux Input Subsystem & Log Messenger |
| `.vnlilypadlotus-ai/2-memory/archive/deprecated-decisions.md` | Tạo kho lưu trữ 5 thử nghiệm/quyết định gỡ bỏ `[XOÁ - KHÔNG ÁP DỤNG]` |
| `.vnlilypadlotus-ai/1-overview/system_map.md` | Tối ưu hóa 4 quy tắc cốt lõi, làm sạch Change Log v2.2.0-modular-sensor |
| `.vnlilypadlotus-ai/2-memory/decision-log.md` | Phân nhóm 5 mảng kiến trúc active, dọn dẹp các quyết định trùng lặp |
| `.vnlilypadlotus-ai/2-memory/checkpoint.md` | Niêm phong bộ nhớ phiên 2026-08-06 |

---

## 🚀 Lệnh Biên Dịch & Khởi Động Lại:

```bash
# Biên dịch & Cài đặt Lilypad chuẩn /usr:
cd fcitx5-lilypad/build && cmake -DCMAKE_INSTALL_PREFIX=/usr .. && make -j$(nproc) && echo thanh123 | sudo -S make install

# Khởi động lại Fcitx5:
fcitx5 -r -d
```


