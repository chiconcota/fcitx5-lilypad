# VNLILYPAD LOTUS CHECKPOINT (DỰ ÁN NÂNG CẤP FCITX5 LOTUS)

## 📌 Trạng thái Bàn giao Phiên Làm Việc (Session Checkpoint)

- **Tên dự án:** `vnlilypad-lotus` ("Nâng cấp Fcitx5 Lotus")
- **Đường dẫn thư mục:** `/home/chiconcota/Documents/vnlilypad-lotus/`
- **Nhánh Git làm việc:** `main` (Remote `origin`: `git@github.com:chiconcota/fcitx5-lilypad.git`)
- **Tình trạng:** **ĐÃ HOÀN THÀNH TÍNH NĂNG SEQUENCER LAYER CHO MODE SEQUENCE (ID 9)**

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc:

1. **Hoàn thiện Sequencer Layer C++ cho `fcitx5-lilypad`:**
   - Tích hợp `Sequencer` class vào [lilypad-sequencer.h](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.h) và [lilypad-sequencer.cpp](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.cpp) với micro-delay 5ms và ACK timeout 35ms.
   - Sửa lỗi thiếu `case LilypadMode::Sequence:` trong các hàm `switch (realMode)` tại [lilypad-state.cpp](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-state.cpp).
2. **Sửa lỗi Backspace Passthrough & Queue Step Order:**
   - Bổ sung `poll_next_step(dummyBs)` trong `performReplacement()` để đưa `CommitString` bước 2 lên đầu hàng đợi ngay khi phát phím xóa.
   - Bổ sung `clear_barrier()` và chuyển lệnh chèn chữ `commitString` qua Fcitx5 Event Loop với hoãn nhịp 5ms.
   - Trả về `return false;` cho phím xóa uinput bay xuyên qua ứng dụng (Passthrough) để thực sự xóa ký tự thô cũ trên màn hình.
3. **Bảo vệ Phím Gõ Nhanh (`replayBufferedKeys`):**
   - Gỡ bỏ rào chắn điều kiện `dbus` trong `replayBufferedKeys()`. Mọi phím gõ nhanh trong lúc xóa (như phím `n` trong `thương`) đều được tái phát lại 100% trên Wayland Native/X11/DBus.
4. **Bảo vệ Bộ Đệm Cho Ứng Dụng Electron/Canvas Block Editors (AFFiNE):**
   - Bổ sung rào chắn `!isFocusOut` trong `LilypadState::reset()` để chặn các sự kiện tái kích hoạt ô nhập liệu liên tục của AFFiNE/Electron không làm xóa bộ đệm gõ Tiếng Việt.
   - **Lưu 2 Phương án triển khai cho phiên kế tiếp:**
     - **Phương án 1 (Anti-Debounce Reset Guard 300ms ~ 500ms):** Nhận diện `App name: AFFiNE` (hoặc các ứng dụng Canvas/Block Editor), đặt ngắt nhịp thời gian khóa lệnh `clearAllBuffers()` và `ResetEngine()` đối với các sự kiện chuyển đổi Input Context có khoảng cách ngắn dưới 300ms~500ms. Chỉ reset khi phím Space/Enter/Tab/Esc hoặc thực sự chuyển cửa sổ.
     - **Phương án 3 (Global Word Buffer Persistence ở tầng `LilypadEngine`):** Chuyển bộ nhớ từ Tiếng Việt (Word Buffer) lên tầng `LilypadEngine` dùng chung thay vì gắn chết theo `LilypadState` của từng `InputContext` riêng lẻ, giúp giữ nguyên ngữ cảnh gõ dù AFFiNE có liên tục tạo và hủy ô nhập liệu 30 lần/giây.

---

## 📁 File Đã Thay Đổi Trong Phiên:

| File | Thay đổi |
| :--- | :--- |
| `fcitx5-lilypad/src/lilypad-sequencer.h` | Thêm class `Sequencer`, `clear_barrier()`, `BarrierState` |
| `fcitx5-lilypad/src/lilypad-sequencer.cpp` | Cài đặt `push_action`, `poll_next_step`, `should_swallow_backspace` |
| `fcitx5-lilypad/src/lilypad-state.cpp` | Sửa switch `LilypadMode::Sequence`, Backspace passthrough, `replayBufferedKeys`, spurious reset guard |
| `fcitx5-lilypad/src/lilypad-engine.cpp` | Ánh xạ Mode `Sequence` (ID 9) trong UI, labels, parsing |
| `fcitx5-lilypad/src/CMakeLists.txt` | Thêm `lilypad-sequencer.cpp` vào danh sách biên dịch |
| `/usr/lib/fcitx5/liblilypad.so` | Thư viện C++ Addon đã biên dịch & cài đặt lên hệ thống |

---

## 🚀 Lệnh Biên Dịch & Cài Đặt Hệ Thống:

```bash
# Biên dịch & Cài đặt Lilypad chuẩn /usr:
cd fcitx5-lilypad/build && cmake -DCMAKE_INSTALL_PREFIX=/usr .. && make -j$(nproc) && echo thanh123 | sudo -S make install

# Khởi động lại Fcitx5:
fcitx5 -r -d
```
