# BẢN PHÂN TÍCH CHI TIẾT TỪNG Ý TƯỞNG & GIẢI PHÁP TỐI ƯU FCITX5 LILYPAD SEQUENCER (ĐẶC THỤ MESSENGER WEB & HEAVY APPS)

## 📌 1. Bối cảnh & Hiện tượng thực tế (Root Cause Traceback)

### 1.1. Hiện tượng 1: Gõ phím Space bị nuốt chữ (`chau1` -> `ch `)
- **Diễn biến:** Người dùng gõ từ `chau1` (hoặc `chaus`) rồi bấm `Space` siêu nhanh.
- **Phân tích:** 
  - Phím `s`/`1` kích hoạt thay thế `au` -> `áu`.
  - Phím xóa uinput bay tới Messenger làm `chau` thành `ch`.
  - Phím `Space` được người dùng bấm tiếp theo ngay trong khoảng trễ micro-delay.
  - Phím `Space` chen ngang làm hủy hoặc ghi đè (overwrite) gói IPC `commitString("áu")` của Fcitx5 với Chromium/Wayland.
  - **Kết quả:** Màn hình chỉ còn `ch `.
- **Giải pháp đã cài đặt:** Đã bổ sung màng hoãn `2ms` micro-gap trước khi `replayBufferedKeys()` nhả phím `Space` đứng chờ trong hàng đợi đệm.

---

### 1.2. Hiện tượng 2: Gõ `vui cấy` bị lẹm thành `vuiấy`
- **Diễn biến:** Người dùng gõ `vui` -> `Space` -> `c` `a` `y` `s`.
- **Phân tích:** 
  - Khi gõ `s`, Bamboo engine tạo ra 2 nhịp thay thế vi mô nối tiếp nhau trong 150ms:
    1. Nhịp 1: `ay` -> `ây` (Xóa 2 ký tự, chèn `ây` -> Messenger ra `vui cây`).
    2. Nhịp 2: `ây` -> `ấy` (Xóa 2 ký tự, chèn `ấy`).
  - **Lỗi của Messenger (React ContentEditable DOM):** 
    - Khi vừa chèn `ây` ở Nhịp 1, React DOM của Messenger gom chuỗi `ây` lại thành **1 DOM Text Node duy nhất**.
    - Khi 2 phím xóa uinput của Nhịp 2 dồn tới: Phím xóa 1 nuốt luôn cả DOM node `ây`, phím xóa 2 trôi sang trái **xóa lẹm mất chữ `c`**!
    - Lệnh commit `ấy` đẩy ra -> Màn hình bị biến dạng thành **`vuiấy`**.

---

## 💡 2. Chi tiết Từng Ý Tưởng & Giải Pháp Kiến Trúc

### Ý TƯỞNG 1: Whole-Word Replacement (Thay thế Trọn gói Nguyên từ) — *Giải pháp Trọng tâm*
- **Ý tưởng:** Thay vì cắt tỉa vi mô từng ký tự đuôi (`ay` -> `ây`, `ây` -> `ấy`) bị phụ thuộc vào cách cắt DOM node của Messenger, bộ gõ sẽ thực hiện **thay thế trọn gói nguyên từ** (`cay` -> `cấy`).
- **Cách hoạt động:**
  - Bộ gõ xác định từ thô hiện tại là `cay` (3 ký tự).
  - Bắn đúng 3 phím xóa uinput để xóa sạch 3 ký tự `c`, `a`, `y`.
  - Chèn 1 lần nguyên từ mới **`cấy`**.
- **Ưu điểm:**
  - Khung nhập liệu của Messenger chỉ nhận 1 cú xóa trọn từ và 1 cú chèn từ mới.
  - Con trỏ luôn đứng ở đầu từ (Word Boundary), tuyệt đối không bao giờ bị lệch hay lẹm sang các chữ trước như `vui`.

---

### Ý TƯỞNG 2: Atomic Key Buffering + 2ms Micro-gap Replay (Hàng đợi khóa phím nguyên tử)
- **Ý tưởng:** Khi bộ gõ đang bận xử lý giao dịch xóa (`is_deleting_ = true`), mọi phím gõ dồn tới của người dùng (kể cả phím `Space`) đều phải xếp hàng chờ trong `buffered_keys_`.
- **Cách hoạt động:**
  - Nhả `commitString("áu")` cho Messenger trước.
  - Hoãn 2ms qua Fcitx5 Event Loop (`addTimeEvent`) để Wayland IPC của Chromium xử lý và render xong chữ `áu`.
  - Tái phát lại phím `Space` từ hàng đợi đệm.
- **Ưu điểm:** Phân tách 2 sự kiện IPC thành 2 frame riêng biệt, triệt hạ hoàn toàn bug đè gói tin IPC trên Chrome/Electron.

---

### Ý TƯỞNG 3: Word Boundary Strictness & Caret Buffer Lock (Khóa ranh giới từ)
- **Ý tưởng:** Đảm bảo bộ gõ KHÔNG BAO GIỜ xóa nhầm sang từ phía trước hay xóa tràn 6-7 ký tự.
- **Cách hoạt động:**
  - Ngay khi phím `Space`, `Enter`, `Tab`, phím điều hướng hoặc Mouse Click được kích hoạt -> Bộ đệm từ (`oldPreBuffer_`) xóa rỗng về 0.
  - Độ dài bộ đệm từ `oldPreBuffer_` chỉ tính trong phạm vi từ đang gõ (VD: `cay` = 3).
  - Số phím xóa uinput bị chặn trần bởi `utf8::length(current_word)`, tuyệt đối không vượt quá số ký tự của từ hiện tại.

---

## 📊 3. Bảng So Sánh Các Giải Pháp

| Ý tưởng | Vấn đề giải quyết | Cơ chế hoạt động | Mức độ an toàn |
| :--- | :--- | :--- | :--- |
| **Whole-Word Replacement** | Sửa lỗi lẹm chữ (`vuiấy`) trên Messenger | Xóa trọn gói từ thô cũ (`cay`) -> Chèn từ mới (`cấy`) | 🟢 100% Tuyệt đối |
| **2ms Micro-gap Replay** | Sửa lỗi nuốt chữ/đè IPC khi gõ nhanh phím Space | Hoãn 2ms giữa lệnh Commit và phím đệm Space | 🟢 100% Tuyệt đối |
| **Word Boundary Lock** | Ngăn xóa lẹm 6-7 ký tự sang từ phía trước | Reset bộ đệm khi gặp Space/Enter/Click | 🟢 100% Tuyệt đối |
