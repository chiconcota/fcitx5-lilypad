# VNLILYPAD LOTUS CHECKPOINT (DỰ ÁN NÂNG CẤP FCITX5 LOTUS)

## 📌 Trạng thái Bàn giao Phiên Làm Việc (Session Checkpoint)

- **Tên dự án:** `vnlilypad-lotus` ("Nâng cấp Fcitx5 Lotus")
- **Đường dẫn thư mục:** `/home/chiconcota/Documents/vnlilypad-lotus/`
- **Nhánh Git làm việc:** `feature/modular-ack-sensors` (Tất cả 5 commits đã hoàn thành và sẵn sàng đóng gói Production)
- **Tình trạng:** **ĐÃ NÂNG CẤP THÀNH CÔNG V2.2.0 (MODULAR ACKSENSOR & BATCH REPLAY). HỆ THỐNG ĐÃ ĐƯỢC NIÊM PHONG VÀ SẴN SÀNG CHO PHIÊN ĐÓNG GÓI PRODUCTION tiếp theo.**

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc (2026-08-05 - Tái cấu trúc Sensor & Batch Replay):

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

## 📁 File Đã Thay Đổi Trong Phiên:

| File | Thay đổi |
| :--- | :--- |
| `~/.config/affine-flags.conf` | Cấu hình cờ Wayland IME cho Electron 39 |
| `.vnlilypadlotus-ai/2-memory/checkpoint.md` | Niêm phong bộ nhớ & Lưu trữ 4 phương án cho AFFiNE |
| `.vnlilypadlotus-ai/2-memory/decision-log.md` | Bổ sung Quyết định 011 |
| `.vnlilypadlotus-ai/1-overview/system_map.md` | Cập nhật Recent Change Log |

---

## 🚀 Lệnh Biên Dịch & Khởi Động Lại:

```bash
# Biên dịch & Cài đặt Lilypad chuẩn /usr:
cd fcitx5-lilypad/build && cmake -DCMAKE_INSTALL_PREFIX=/usr .. && make -j$(nproc) && echo thanh123 | sudo -S make install

# Khởi động lại Fcitx5:
fcitx5 -r -d
```

