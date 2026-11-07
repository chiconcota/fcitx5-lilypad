# KẾ HOẠCH TỐI ƯU HÓA CACHE & ĐỒNG BỘ VĂN BẢN LỚN (PHƯƠNG ÁN B)
@target: fcitx5-lilypad Core Engine | @version: v2.3.15 | @status: Implemented & Compiled

## 1. MỤC TIÊU CỐT LÕI (CORE OBJECTIVES)
1. **Triệt tiêu hiện tượng nuốt phím / lặp chữ khi Paste văn bản lớn:** Đảm bảo khi người dùng dán (Ctrl+V) hàng nghìn từ vào ô chat Antigravity IDE hoặc tài liệu AFFiNE, bộ gõ vẫn duy trì tốc độ gõ dấu tức thì, không bị lệch pha con trỏ hay gửi thừa Backspace.
2. **Loại bỏ vòng lặp ngủ nghẽn luồng (Blocking Retry Loop):** Không cho phép Fcitx5 gọi `sleep_for` trên main event loop khi `realtextLen` không khớp với `surr.cursor()`.
3. **Giới hạn phạm vi quét Autofill (Bounded Window Scan):** Chỉ tìm kiếm `oldPreBuffer_` trong phạm vi hẹp cục bộ (ngay trước con trỏ $\le 16$ ký tự), cấm quét toàn bộ chuỗi $10.000$ ký tự gây match nhầm từ xa.
4. **Sửa dứt điểm lỗi rã từ / rơi phím trong `replayBufferedKeys`:** Bảo toàn trạng thái từ đang gõ trong bộ đệm khi Sequencer giải phóng barrier, không để Bamboo bị chuyển sang `english_bypass` sinh ra số thô VNI (`phuo7g`, `o73d9au6`).

---

## 2. NGUYÊN NHÂN & ĐIỂM CHẠM MÃ NGUỒN CẦN XỬ LÝ

| Điểm chạm | Vị trí mã nguồn | Vấn đề hiện tại | Giải pháp tối ưu |
| :--- | :--- | :--- | :--- |
| **1. Quét chuỗi toàn tài liệu** | `lilypad-state.cpp:162`<br>`text.find(oldPreBuffer_)` | Quét từ vị trí $0$ của văn bản $10.000$ chữ. Nếu từ vừa gõ trùng với từ ở đầu tài liệu, hệ thống tưởng nhầm là trình duyệt đang Autofill $\to$ gửi thừa 1 Backspace xóa lẹm chữ. | Chỉ cắt một chuỗi con cục bộ xung quanh con trỏ: `std::string_view localWindow = text.substr(cursor - 16, 16)` để so sánh. |
| **2. Lệch pha con trỏ sau khi Paste** | `lilypad-state.cpp:481`<br>`realtextLen != surr.cursor()` | Khi Paste, `realtextLen` không biết có bao nhiêu ký tự được dán $\to$ ép luồng chạy `sleep_for(2ms)` $\times 3$ lần gây đơ luồng và nuốt phím. | Cập nhật ngay `realtextLen = surr.cursor()` khi phát hiện khoảng nhảy con trỏ lớn (Paste event), loại bỏ hoàn toàn `sleep_for` vô điều kiện. |
| **3. Xóa sổ engine trong hàng đợi phím** | `lilypad-state.cpp:1674`<br>`replayBufferedKeys()` | Sau khi phát lại phím đệm, code lại gọi `ResetEngine()` và `oldPreBuffer_.clear()`, làm Bamboo mất sạch ngữ cảnh từ đang gõ $\to$ rã dấu thành số thô. | Giữ nguyên trạng thái `oldPreBuffer_` và Bamboo context để tiếp tục ghép âm tự nhiên. |
| **4. Cửa sổ giữ cờ xóa quá dài (70ms)** | `lilypad-state.cpp:562`<br>`settle_delay_us = 70ms` | Cờ `is_deleting_ = true` bị giữ quá lâu trên Chromium/Electron (70ms), phím gõ nhanh tiếp theo (`n`) dễ bị rơi hoặc đệm chậm. | Hạ trần Settling Window thích ứng theo tốc độ gõ IKI thực tế (tối ưu xuống 20ms - 30ms). |
| **5. Treo buffer cũ khi Focus Bounce** | `lilypad-engine.cpp:410`<br>`Focus bounce keep word buffers` | Focus Bounce ở AFFiNE giữ lại buffer vô thời hạn cả khi đã dừng gõ từ cũ, khiến từ mới bị dán dính vào từ cũ. | Chỉ giữ buffer khi thực sự có giao dịch xóa vi mô đang diễn ra (`is_deleting_ == true`). Khi nhàn rỗi, phải reset sạch bộ đệm. |

---

## 3. LỘ TRÌNH TRIỂN KHAI THEO CÁC BƯỚC (STEP-BY-STEP PLAN)

### Bước 1: Thu hẹp phạm vi quét `isAutofillCertain` (Bounded Window)
- **Hành động:** Thay đổi cách trích xuất văn bản trong `isAutofillCertain(const SurroundingText& s)`:
  - Xác định vị trí con trỏ `cursor = s.cursor()`.
  - Chỉ trích xuất tối đa 16 ký tự ngay trước con trỏ:
    $$\text{windowStart} = \max(0, \text{cursor} - \text{buffLen} - 4)$$
  - Chỉ thực hiện kiểm tra `sameprefix` trên lát cắt `window` này. Không bao giờ duyệt qua $10.000$ ký tự của tài liệu.

### Bước 2: Tự động đồng bộ `realtextLen` & Khử triệt để Blocking Sleep
- **Hành động:**
  - Trong `lilypad-state.cpp`, khi nhận sự kiện phím hoặc `surroundingText`:
    - Nếu khoảng cách $|\text{surr.cursor()} - \text{realtextLen}| > 4$, đánh dấu đây là thao tác di chuyển con trỏ hoặc Paste văn bản lớn.
    - Lập tức đồng bộ `realtextLen.store(surr.cursor())` và xóa rỗng bộ đệm cũ an toàn (`ResetEngine()`, `oldPreBuffer_.clear()`).
  - Trong `handleUInputKeyPress`: Loại bỏ vòng lặp `sleep_for(2ms)` gây lag main thread, thay bằng cơ chế non-blocking event hoặc bỏ qua nếu đã ở Sequence Mode.

### Bước 3: Sửa lỗi bảo toàn từ trong `replayBufferedKeys`
- **Hành động:**
  - Sửa dòng 1673-1676 trong `lilypad-state.cpp`:
    - Sau khi `performReplacement(deletedPart, addedPart)`, cập nhật `oldPreBuffer_ = preeditStr` thay vì gọi `ResetEngine()` và `oldPreBuffer_.clear()`.
    - Đảm bảo từ đang gõ dở (như `phương`) tiếp tục nhận các phụ âm tiếp theo một cách trơn tru.

### Bước 4: Tinh chỉnh Post-Commit Settling Window thích ứng (20ms – 30ms)
- **Hành động:**
  - Trong `calculateDynamicPostSettlingUs()`:
    - Nếu người dùng đang gõ ở nhịp nhanh ($< 200\text{ms}$), giảm trần trễ hậu commit từ 70ms xuống còn $25\text{ms}$.
    - Giúp giải phóng cờ `is_deleting_` nhanh hơn, không làm nghẽn các phím tiếp theo như `n` hay `g`.

### Bước 5: Kiểm thử thực tế (Verification)
- **Test Case 1:** Dán một đoạn văn bản $5.000 - 10.000$ từ vào ô chat Antigravity IDE $\to$ Gõ liên tục các từ phức tạp VNI: `phương`, `thưởng`, `việt nam`, `bắc` $\to$ Xác nhận không bị nuốt phím `n`, không lặp chữ, không xuất số thô.
- **Test Case 2:** Dán đoạn văn bản lớn vào AFFiNE $\to$ Gõ dấu tại các vị trí đầu, giữa và cuối đoạn văn $\to$ Xác nhận không bị xóa lẹm sang các từ đã paste trước đó.

---

## 4. GHI NHẬN HIỆN TRƯỜNG & BÀN GIAO PHIÊN TIẾP THEO (2026-11-05)
- **Kiểm thử thực tế qua video (`media_1793884389697_9096f080.mp4`):**
  - Người dùng dán văn bản vào ô chat Antigravity IDE rồi gõ: `b gõ bị lo` $\to$ chữ `lo` bị xóa mất hoàn toàn khi bỏ dấu, và các phím sau rơi rụng thành `n b !`.
  - Người dùng nhận định nguyên nhân gốc rễ có thể nằm ở khía cạnh khác thay vì Safety Timer / Race Condition.
  - Tạm dừng phiên làm việc theo yêu cầu người dùng để phiên sau tiếp tục khảo sát toàn diện. Mã nguồn v2.3.15 được giữ nguyên vẹn và sạch sẽ.

