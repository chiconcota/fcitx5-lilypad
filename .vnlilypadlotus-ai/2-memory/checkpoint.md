# VNLILYPAD LOTUS CHECKPOINT (DỰ ÁN NÂNG CẤP FCITX5 LOTUS)

## 📌 Trạng thái Bàn giao Phiên Làm Việc (Session Checkpoint)

- **Tên dự án:** `vnlilypad-lotus` ("Nâng cấp Fcitx5 Lotus")
- **Đường dẫn thư mục:** `/home/chiconcota/Documents/vnlilypad-lotus/`
- **Nhánh Git làm việc:** `main` (Remote `origin`: `git@github.com:chiconcota/fcitx5-lilypad.git`)
- **Tình trạng:** **HOÀN THÀNH TỐI ƯU CẢM BIẾN ACK NĂNG ĐỘNG (250MS) & XÓA VI MÔ CHUẨN XÁC TRÊN MESSENGER / FACEBOOK WEB**

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc (2026-08-05):

1. **Sửa lỗi Cảm biến ACK Thích ứng (`calculate_adaptive_delay_ms`):**
   - Kích hoạt `calculate_adaptive_delay_ms(elapsed)` trong `Sequencer::receive_ack()` ở [lilypad-sequencer.cpp](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.cpp#L63) và gán `last_measured_ack_ms_ = adaptive`.
   - Nâng trần Safety Timeout `max_ack_timeout_ms` từ `35ms` lên **`250ms`** trong [lilypad-sequencer.h](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.h#L38). Khi ứng dụng lag đến 159ms, rào chắn tự động nới rộng ra 150ms~250ms chờ app render xong 100%, triệt hạ lỗi nuốt mất chữ `thươn`.
2. **Tối ưu hóa Xóa vi mô (Micro-replacement) & Caret Buffer Lock Cap:**
   - Chuẩn hóa luồng `performReplacement` trong [lilypad-state.cpp](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-state.cpp#L513-L565) về phím xóa vi mô tối ưu (`utf8::length(deletedPart)`). Khi `o` -> `ơ`, chỉ phát đúng 1 phím xóa; khi `uơ` -> `ươn`, chỉ phát đúng 2 phím xóa.
   - Triệt hạ lỗi đơ React DOM Facebook làm dính chữ rác `tthuo7n`.
   - Giữ vững rào chắn khống chế trần phím xóa `bsCount <= utf8::length(oldPreBuffer_)` tuyệt đối không cho phím xóa bay quá phím Space sang từ phía trước.
3. **Chuẩn hóa Replay Reset:**
   - Đã bổ sung `ResetEngine()` và `oldPreBuffer_.clear()` cho nhánh `!processed` trong `replayBufferedKeys()` để phím Space/điều hướng nhả từ hàng đợi đệm luôn lập ranh giới từ mới sạch sẽ.

---

## 📁 File Đã Thay Đổi Trong Phiên:

| File | Thay đổi |
| :--- | :--- |
| `fcitx5-lilypad/src/lilypad-sequencer.h` | Nâng `max_ack_timeout_ms` từ 35ms lên 250ms |
| `fcitx5-lilypad/src/lilypad-sequencer.cpp` | Kích hoạt `calculate_adaptive_delay_ms(elapsed)` trong `receive_ack()` và gán `last_measured_ack_ms_ = adaptive` |
| `fcitx5-lilypad/src/lilypad-state.cpp` | Micro-replacement optimization, Caret Buffer Lock Cap, Replay Reset logic |
| `.vnlilypadlotus-ai/1-overview/system_map.md` | Cập nhật Recent Change Log & trạng thái module |
| `.vnlilypadlotus-ai/2-memory/decision-log.md` | Bổ sung Quyết định 010 |
| `.vnlilypadlotus-ai/2-memory/checkpoint.md` | Niêm phong bộ nhớ phiên làm việc |
| `fcitx5-lilypad/src/lilypad-engine.cpp` | Ánh xạ Mode `Sequence` (ID 9) trong UI, labels, parsing |
| `fcitx5-lilypad/src/CMakeLists.txt` | Thêm `lilypad-sequencer.cpp` vào danh sách biên dịch |
| `/usr/lib/fcitx5/liblilypad.so` | Thư viện C++ Addon đã biên dịch & cài đặt lên hệ thống |
| `/usr/bin/fcitx5-lilypad-server` | Server daemon đã biên dịch & cài đặt lên hệ thống |

---

## 🚀 Lệnh Biên Dịch & Cài Đặt Hệ Thống:

```bash
# Biên dịch & Cài đặt Lilypad chuẩn /usr:
cd fcitx5-lilypad/build && cmake -DCMAKE_INSTALL_PREFIX=/usr .. && make -j$(nproc) && echo thanh123 | sudo -S make install

# Khởi động lại Fcitx5:
fcitx5 -r -d
```
