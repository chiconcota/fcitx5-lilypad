# BÁO CÁO PHÂN TÍCH TOÀN DIỆN & CÁC GIẢI PHÁP TRIỆT ĐỂ: HIỆN TƯỢNG RƠI PHÍM DO FOCUS BOUNCE TRÊN WAYLAND (CHROMIUM / ELECTRON)

@target: `fcitx5-lilypad` Core Architecture  
@topic: Triệt tiêu hiện tượng rụng phím vật lý khi Chromium nảy tiêu điểm (Focus Bounce) trên Wayland Compositor (Niri)  
@status: Architecture Investigation & Solution Proposals  
@date: 2026-11-06  

---

## 📌 1. BẢN CHẤT VẤN ĐỀ & BẰNG CHỨNG KỸ THUẬT

### 1.1 Hiện tượng thực tế
Người dùng gõ từ có biến âm hoặc kết thúc từ bằng VNI/Telex trên Google Chrome / TikTok Web / AFFiNE:
* **Ví dụ:** Gõ từ `thương` theo kiểu VNI (`t - h - u - o - 7 - n - g - Space`).
* **Kết quả hiển thị trên màn hình:** Bị biến thành **`thuo7n`** (thiếu hẳn ký tự `g`).
* **Phân tích log hệ thống:**
  ```text
  I2026-11-06 12:01:40.430690 ... 🎯 [DYNAMIC GOVERNOR] Recorded Focus Bounce event for context
  I2026-11-06 12:01:40.465726 ... 🎯 [FOCUS SETTLED] Chromium focus bounce settled after 84ms. Executing commit safely!
  I2026-11-06 12:01:40.465783 ... ⚡ [SEQUENCER EXECUTE] Dispatching MicroStep [Serial #90]: type=2 text=uo7n
  I2026-11-06 12:01:40.475831 ... Commit (Sequence Mode): uo7n
  ```

### 1.2 Cơ chế dây chuyền gây lỗi (Chain of Causation)
1. **Phím xóa kích hoạt DOM Reflow:**
   Tại phím `7` (hoặc `w`), bộ gõ Bamboo thay thế `o` $\to$ `uơ`. Fcitx5 gửi phím Backspace qua `/dev/uinput` để xóa ký tự cũ.
2. **Chromium giật tiêu điểm (Focus Bounce):**
   Trong các trình soạn thảo nền web (React / Draft.js / Lexical / ProseMirror), khi xóa ký tự, DOM node cũ bị unmount và node mới được gắn vào. Trong tích tắc node cũ bị hủy, `document.activeElement = null`.
   Trình duyệt Chromium lập tức gửi yêu cầu giao thức Wayland: `zwp_text_input_v3::disable`.
3. **Compositor (Niri) nhả Keyboard Grab:**
   Theo đặc tả bắt buộc của chuẩn Wayland, khi nhận lệnh `disable`, Niri phải hủy bỏ quyền giữ phím (Keyboard Grab) của Fcitx5. Toàn bộ phím phần cứng từ bàn phím lúc này được chuyển thẳng sang cửa sổ ứng dụng qua `wl_keyboard`.
4. **Phím vật lý tiếp theo (`g`) bị rơi vào khoảng mù:**
   Người dùng gõ phím `g` với tốc độ tự nhiên ($\approx 80\text{ms} \sim 100\text{ms}$). Phím `g` rơi trúng vào lúc cánh cửa Grab đang đóng:
   * Fcitx5 **hoàn toàn không nhận được tín hiệu phím** từ Niri (log không hề có dòng `Typing so fast, add key to queue`).
   * Chromium nhận được phím `g` thô qua `wl_keyboard` nhưng vì node DOM đang unmount/re-render nên Chromium lờ đi hoặc vứt bỏ.
5. **Auto-Restore của Bamboo kích hoạt:**
   Khi người dùng bấm Space kết thúc từ, Bamboo Core chỉ nhận được `thươn`. Do `thươn` không có trong từ điển tiếng Việt, cơ chế `AutoNonVnRestore` tự động hoàn tác chuỗi phím thô: `th` + `uo7n` = **`thuo7n`**.

---

## 🧪 2. TỔNG HỢP CÁC PHƯƠNG ÁN ĐÃ TỪNG THỬ NGHIỆM & KẾT QUẢ THỰC TẾ

Dưới đây là đối soát chi tiết toàn bộ các phương án kỹ thuật đã từng được nghiên cứu, lập trình và kiểm nghiệm thực địa trên `fcitx5-lilypad`:

### ❌ Thử nghiệm 1: Commit Sớm Bằng Safety Timer (`barrier_safety_timer_` 20ms – 60ms)
* **Ý tưởng:** Khi phát Backspace + Sentinel `KEY_F24`, nếu F24 không quay về sau 20ms ~ 40ms, Safety Timer tự động kích hoạt và commit chuỗi chữ mới vào app.
* **Kết quả thực tế:** **Thất bại nặng nề (Gây thảm họa cụt chữ / mất dấu)**.
  - Trên Chromium/TikTok/AFFiNE, DOM reflow làm các phím Backspace từ uinput bị delay tới $50\text{ms} \sim 80\text{ms}$ mới đến nơi.
  - Khi timer tự ý commit ở mốc 20ms, các phím Backspace dội về sau xóa sạch chữ vừa commit, gây cụt chữ (`miếng` $\to$ `m`, `cháu` $\to$ `ch`).
  - **Kết luận:** Đã bị khai tử hoàn toàn tại Quyết định 037.

---

### ❌ Thử nghiệm 2: Tắt Extension Wayland TextInputV3 Của Chrome (`--disable-features=WaylandTextInputV3`)
* **Ý tưởng:** Khởi động Google Chrome với cờ tắt `text-input-v3` để Chrome không gửi tín hiệu `enable`/`disable` lên Wayland Compositor.
* **Kết quả thực tế:** **Thất bại hoàn toàn**.
  - Chromium trên Wayland bị tê liệt hoàn toàn khả năng nhập liệu, con trỏ chuột và bàn phím bị đơ cứng, không thể gõ được bất kỳ văn bản nào.
  - **Kết luận:** Bị loại bỏ ngay lập tức sau thử nghiệm.

---

### ⚠️ Thử nghiệm 3: Chờ Độc Quyền Phím Phần Cứng Sentinel `KEY_F24` (Khai Tử Commit Sớm)
* **Ý tưởng:** Không cho phép bất kỳ timer nào commit trước. Duy nhất phím Sentinel `KEY_F24` dội về Fcitx5 mới được phép kích hoạt commit.
* **Kết quả thực tế:**
  - *Mặt tốt:* Hoạt động hoàn hảo và mượt mà tuyệt đối trên 95% ứng dụng Native (Ghostty, VS Code, GNOME Text Editor — phản hồi 1.5ms).
  - *Mặt xấu trên Chrome:* Uinput phát `KEY_F24` ở $t=3\text{ms}$ đúng vào microsecond Chrome đang gửi `disable`. Niri nhả Grab làm `KEY_F24` bị ném sang Chrome thay vì Fcitx5 $\implies$ Fcitx5 bị treo cứng chờ đến khi nổ $300\text{ms}$ Watchdog Timeout mới cắt lỗ khẩn cấp.
  - **Kết luận:** Cần cơ chế cứu hộ riêng cho Chromium khi F24 bị Compositor làm rơi.

---

### ❌ Thử nghiệm 4: Hạ Post-Commit Settling Window Xuống 12ms (Phiên bản v2.3.9)
* **Ý tưởng:** Hạ thời gian giam phím Post-Commit từ 70ms xuống 12ms để giải phóng phím siêu tốc sau khi commit.
* **Kết quả thực tế:** **Thất bại (Gãy từ có biến âm kép)**.
  - Gõ các từ có thay thế kép liên tiếp như `thay` + `a` $\to$ `thây` + `s` $\to$ `thấy` hoặc `thương` bị rụng sạch phần đuôi `ấy`, chỉ còn lại `th`.
  - Chromium/Electron (đặc biệt là Lexical, Canvas của AFFiNE) cần thời gian để reconciliation DOM tree cho commit trước trước khi nhận một đợt Backspace tiếp theo.
  - **Kết luận:** Bắt buộc phải khôi phục mức sàn an toàn 70ms ở v2.3.10.

---

### ⚠️ Thử nghiệm 5: Ý Tưởng Bắn Thêm Phím Vô Hại F23 Để Giữ Focus (Thảo luận kỹ thuật)
* **Ý tưởng:** Sau phím xóa cuối cùng, bắn thêm 1 phím vô hại F23 để ép giữ focus không bị FocusOut.
* **Kết quả phân tích:** **Không khả thi**.
  - Nguyên nhân Chrome FocusOut là do React/Blink unmount DOM node trong trình duyệt (`document.activeElement = null`), hoàn toàn không liên quan đến việc có phím bấm vật lý hay không.
  - **Kết luận:** Không giải quyết được gốc rễ.

---

### 🟢 Thử nghiệm 6: Ghim Tiêu Điểm Ảo (Virtual Focus Pinning tại `LilypadEngine::deactivate()`)
* **Ý tưởng:** Bỏ qua 100% sự kiện FocusOut ảo khi `is_deleting_ == true` hoặc bộ đệm còn chữ `!oldPreBuffer_.empty()`.
* **Kết quả thực tế:** **Thành công bảo vệ bộ nhớ nội bộ của Fcitx5**.
  - Fcitx5 không tự hủy buffer, không xóa input panel và không reset Bamboo Core khi Chrome nảy focus.
  - *Hạn chế:* Chỉ có tác dụng nội bộ trong tiến trình Fcitx5; không thể ngăn Niri Compositor nhả Keyboard Grab ở tầng Wayland bên ngoài.
  - **Kết luận:** Giữ lại làm nền tảng bảo vệ vững chắc trong mã nguồn.

---

### 🟡 Thử nghiệm 7: Focus Settled Debounce Lắng Dịu 35ms (v2.4.6 - Đang hoạt động)
* **Ý tưởng:** Khi phát hiện Focus Bounce dồn dập trên Chrome, đặt bộ đếm lắng dịu 35ms sau nhịp nảy cuối cùng mới chốt commit và đo telemetry thực tế.
* **Kết quả thực tế:**
  - *Thành công:* Giải cứu giao dịch hoàn toàn, triệt tiêu $100\%$ lỗi kẹt 300ms Watchdog (`miếng` ra chuẩn, đo trễ thực tế 84ms).
  - *Phát sinh điểm nghẽn mới:* Trong 84ms Chrome nảy focus, nếu người dùng gõ phím tiếp theo (ví dụ phím `g` trong VNI `thuo7ng`), phím `g` bị Niri nhả Grab lọt vào khoảng mù $\implies$ Bamboo nhận thiếu `g` thành `thươn` $\implies$ Bamboo AutoRestore thành `thuo7n`.
  - **Kết luận:** Đã giải quyết được bài toán Commit & Telemetry, nhưng cần bước tiếp theo để bảo vệ phím gõ thô không bị Niri vứt đi.

---

## 🚀 3. TỔNG HỢP CÁC PHƯƠNG ÁN GIẢI QUYẾT TRIỆT ĐỂ (KẾ HOẠCH PHIÊN TỚI)

---

### 🟢 GIẢI PHÁP 1: Lưới Bảo Hiểm Phần Cứng Từ Daemon Server (Libinput Hardware Tap)
> **Độ tin cậy:** ⭐⭐⭐⭐⭐ ($100\%$ không thể rơi phím)  
> **Độ phức tạp kiến trúc:** Trung bình – Khá  
> **Trải nghiệm người dùng:** Giữ nguyên 100% Sequence Mode tự nhiên, không gạch chân.

#### Nguyên lý vận hành:
* Tiến trình daemon `/usr/bin/fcitx5-lilypad-server` chạy với quyền root/uinput và đã mở sẵn một context **`libinput`** kết nối trực tiếp tới `seat0` (hiện tại đang dùng để bắt sự kiện chuột).
* Server có thể mở rộng để lắng nghe thêm sự kiện bàn phím phần cứng (`LIBINPUT_EVENT_KEYBOARD_KEY`).
* **Cơ chế hoạt động:**
  1. Khi Fcitx5 bắt đầu giao dịch xóa (`is_deleting_ = true`), Fcitx5 gửi tín hiệu bật "chế độ rào chắn" cho server qua Unix Domain Socket (`kb_socket`).
  2. Nếu người dùng gõ bất kỳ phím nào trong lúc này (như phím `g`):
     - Server đọc trực tiếp từ nhân Linux `/dev/input/event*`, **hoàn toàn đứng bên ngoài và bên dưới Niri Compositor**.
     - Server lập tức gửi mã phím đó về cho Fcitx5 qua socket.
  3. Fcitx5 nhận phím từ Server và đẩy thẳng vào hàng đợi RAM `buffered_keys_`.
  4. Ngay khi Focus Settled kết thúc, Fcitx5 xả sạch hàng đợi và chuyển tiếp cho Bamboo Core.
* **Ưu điểm:**
  - Vượt qua giới hạn giao thức của Wayland: Dù Niri có nhả Grab hay Chrome có FocusOut bao nhiêu lần thì Server phần cứng vẫn tóm gọn từng phím gõ.
  - Chữ `g` được bảo tồn $100\%$.
* **Thách thức:** Cần cơ chế khử trùng lặp (de-duplication) nếu Niri bất ngờ gửi phím đó cùng lúc với Server.

---

### 🟡 GIẢI PHÁP 2: Điều Tốc Xả Phím Xóa & Bắt Tay Từng Bước (Paced Backspace with Barrier ACK)
> **Độ tin cậy:** ⭐⭐⭐⭐  
> **Độ phức tạp kiến trúc:** Thấp  
> **Trải nghiệm người dùng:** Êm ái, giảm thiểu tối đa hiện tượng nảy Focus của Chromium.

#### Nguyên lý vận hành:
* Hiện tại trong `lilypad-server.cpp`, các phím Backspace được xả liên thanh với chu kỳ $1\text{ms}$ (`poll_timeout = 1`).
* Việc nã liên thanh $3 \sim 4$ phím Backspace trong vòng $3\text{ms}$ làm Blink thread của Chrome bị "nghẹn", dẫn tới việc React phải liên tục hủy và vẽ lại DOM tới 6 lần.
* **Cải tiến:**
  - Thay vì xả ồ ạt $1\text{ms}$, server xả phím xóa có nhịp thở (Pacing) $\approx 8\text{ms} \sim 12\text{ms}$ mỗi phím đối với các ứng dụng có cờ `wa_chromium_flag`.
  - Hoặc sau mỗi phím xóa, kiểm tra nhịp ổn định của compositor trước khi bắn phím xóa tiếp theo.
* **Ưu điểm:** Giảm tải xung đột DOM trong Chromium, thu hẹp cửa sổ Focus Bounce từ $84\text{ms}$ xuống dưới $20\text{ms}$.
* **Nhược điểm:** Tăng nhẹ tổng thời gian xóa đối với các từ cần xóa nhiều ký tự ($N \ge 3$).

---

### 🔵 GIẢI PHÁP 3: Cơ Chế Composition Preedit Thích Ứng Cho Heavy Webview
> **Độ tin cậy:** ⭐⭐⭐⭐⭐ ($100\%$ không có Focus Bounce)  
> **Độ phức tạp kiến trúc:** Rất thấp  
> **Trải nghiệm người dùng:** Xuất hiện gạch chân tạm thời khi đang soạn thảo từ trên Chrome/TikTok.

#### Nguyên lý vận hành:
* Trong `fcitx5-lilypad`, chế độ `Preedit` sử dụng giao thức chuẩn `zwp_input_method_v2::set_preedit_string`.
* Khi soạn thảo từ, chữ được vẽ qua lớp Composition của Wayland (có đường gạch chân), **hoàn toàn không phát sinh một phím Backspace nào qua uinput**.
* Khi kết thúc từ (Space / Enter / phím điều hướng), từ hoàn chỉnh mới được đưa vào DOM bằng 1 lệnh `commitString`.
* **Cải tiến:**
  - Cho phép người dùng gán quy tắc theo ứng dụng (`ack_apps` hoặc cấu hình riêng): Tự động chuyển sang chế độ Preedit khi con trỏ nằm trên các trang web có DOM nặng (TikTok Web, AFFiNE, Google Docs).
* **Ưu điểm:** Triệt tiêu $100\%$ hiện tượng nảy Focus, không cần uinput, độ trễ $0\text{ms}$.
* **Nhược điểm:** Từ có gạch chân trong lúc gõ (không phải dạng Sequence gõ trực tiếp thuần túy).

---

### 🟣 GIẢI PHÁP 4: Hoàn Thiện Định Tuyến Hàng Đợi Phím Xuyên Context (`g_active_deleting_state`)
> **Độ tin cậy:** ⭐⭐⭐  
> **Độ phức tạp kiến trúc:** Thấp  
> **Trải nghiệm người dùng:** Khắc phục trường hợp phím không bị Niri vứt đi mà bị rơi vào context phụ `ctx_...`.

#### Nguyên lý vận hành:
* Trong log xuất hiện `ctx_0x5633d85f4410` ngay sau giao dịch commit.
* Khi Chrome nảy focus, Wayland tạo ra hoặc trỏ tới context phụ `ctx_...`. Hiện tại, trong `LilypadState::keyEvent()`, khi `is_deleting_` đang bật:
  ```cpp
  buffered_keys_.push_back({.sym = currentSym, ...});
  ```
  Nó chỉ đẩy vào `buffered_keys_` của context hiện tại (`this`).
* **Cải tiến:**
  - Nếu `g_active_deleting_state` đang trỏ tới một context khác (context chính), mọi phím gõ nhanh trong lúc `is_deleting_` **bắt buộc phải được đẩy vào hàng đợi của `g_active_deleting_state`**.
  - Khi context chính kết thúc chu trình settling, nó sẽ xả trọn vẹn toàn bộ các phím được gõ dồn vào.
* **Ưu điểm:** Đảm bảo không một phím nào bị "kẹt lại" ở các context phụ.

---

## 📊 4. BẢNG SO SÁNH TỔNG HỢP CÁC PHƯƠNG ÁN

| Phương án | Nguyên lý cốt lõi | Ưu điểm lớn nhất | Nhược điểm / Rủi ro | Mức độ ưu tiên |
| :--- | :--- | :--- | :--- | :--- |
| **Giải pháp 1 (Server Libinput Tap)** | Server đọc phím vật lý độc lập với Niri | Bảo vệ phím $100\%$ không thể bị drop | Cần xử lý khử trùng lặp phím khi Niri mở lại Grab | **Khuyến nghị số 1 cho Phiên sau** |
| **Giải pháp 4 (Cross-Context Queue)** | Định tuyến mọi phím về context đang xóa | Đơn giản, giải quyết lỗi chia rẽ bộ nhớ RAM | Không cứu được nếu phím bị Niri vứt ra ngoài | **Triển khai song song với GP 1** |
| **Giải pháp 2 (Paced Backspace)** | Giãn nhịp xả phím xóa ($8\text{ms} \sim 12\text{ms}$) | Giảm nghẽn DOM cho Chromium | Tăng nhẹ thời gian xóa từ dài | Cân nhắc kết hợp |
| **Giải pháp 3 (Preedit Rule)** | Dùng Wayland Preedit cho app cực nặng | 0ms trễ, triệt tiêu $100\%$ nảy focus | Chữ có gạch chân tạm thời | Giải pháp tùy chọn cho người dùng |

---

## 🎯 5. KẾ HOẠCH BÀN GIAO CHO PHIÊN LÀM VIỆC TIẾP THEO

1. **Thảo luận & Lựa chọn giải pháp:**
   - Đánh giá tính khả thi của **Giải pháp 1 (Lưới bảo hiểm phần cứng từ Server)** kết hợp **Giải pháp 4 (Định tuyến hàng đợi phím xuyên Context)**.
2. **Kế hoạch triển khai mã nguồn:**
   - Cập nhật `fcitx5-lilypad-server`: Đăng ký lắng nghe sự kiện phím bàn phím qua `libinput` và tạo socket chuyển tiếp sự kiện phím cứu trợ.
   - Cập nhật `lilypad-state.cpp` / `lilypad-state-governor.cpp`: Hứng các phím cứu trợ từ server khi `is_deleting_ == true`.
3. **Kiểm thử đối soát:**
   - Test gõ tốc độ cao các từ VNI có dấu móc/nón kép (`thương`, `thưởng`, `thấy`, `nghiêng`) trên TikTok Web và AFFiNE.

