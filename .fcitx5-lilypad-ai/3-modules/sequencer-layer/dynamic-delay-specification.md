# ĐẶC TẢ CÔNG THỨC TOÁN HỌC & HỆ THỐNG ĐẠI LƯỢNG VI TRỄ ĐỘNG (FCITX5 LILYPAD)

@module: `Sequencer Layer` (`fcitx5-lilypad/src/ack-sensors/`, `lilypad-state.cpp`, `lilypad-sequencer.cpp`)  
@target: Kiến trúc Điều hòa Vi trễ Tự thích ứng (Adaptive Dynamic Micro-Pacing)  
@status: Specification & Architecture Reference  

---

## 📌 1. BẢNG ĐỊNH NGHĨA TOÀN BỘ CÁC ĐẠI LƯỢNG & ĐƠN VỊ ĐO

| Ký hiệu | Tên đại lượng | Đơn vị đo | Kiểu dữ liệu (C++) | Ý nghĩa & Bản chất vật lý |
| :--- | :--- | :--- | :--- | :--- |
| **$N$** | Số phím xóa ký tự thô | *Không đơn vị* (số nguyên) | `int` | Số ký tự thực tế cần xóa trong bộ đệm (`utf8::length(deletedPart)`). Ví dụ: `o` $\to$ `ơ` ($N=1$); `uo` $\to$ `ươ` ($N=2$). |
| **$N + 1$** | Tổng số phím xóa uinput phát đi | *Không đơn vị* (số nguyên) | `int` | $N$ phím đầu cho đi vào App để xóa text + 1 phím cuối (Sentinel) để Fcitx5 nuốt và chốt thời điểm an toàn. |
| **$T_1$** | Mốc thời gian phát lệnh xóa | Thời điểm (`time_point`) | `steady_clock::time_point` | Mốc thời gian bấm giờ ngay trước khi gọi `send_backspace_uinput(N + 1)`. |
| **$T_2$** | Mốc thời gian nuốt xong Sentinel | Thời điểm (`time_point`) | `steady_clock::time_point` | Mốc thời gian chốt giờ ngay khi phím Sentinel thứ $N+1$ quay về Fcitx5 (`handleUInputKeyPress`). |
| **$\Delta T_{\text{swallow}}$** | Tổng thời gian nuốt phím xóa | Microsecond ($\mu\text{s}$) / Millisecond ($\text{ms}$) | `int64_t` | $\Delta T_{\text{swallow}} = T_2 - T_1$. Thời gian thực tế để $N+1$ phím hoàn thành vòng lặp: `/dev/uinput` $\to$ Kernel $\to$ Compositor $\to$ Fcitx5. |
| **$T_{\text{measured}}$** | Thời gian nuốt trung bình mỗi phím | Microsecond ($\mu\text{s}$) | `uint64_t` | $T_{\text{measured}} = \frac{\Delta T_{\text{swallow}}}{N + 1}$. Phản ánh độ trễ xử lý thô của 1 phím xóa trong đúng lần gõ này. |
| **$\text{app\_ack}$** | Độ trễ nuốt phím làm mịn qua EMA | Microsecond ($\mu\text{s}$) | `std::atomic<uint64_t>` | Giá trị trung bình trượt có trọng số của $T_{\text{measured}}$, triệt tiêu xung nhiễu giật cục của CPU/hệ thống. |
| **$\text{IKI}$** | Nhịp gõ ngón tay (Inter-Keystroke Interval) | Millisecond ($\text{ms}$) | `uint64_t` | Khoảng thời gian ngón tay con người gõ giữa 2 phím vật lý liên tiếp ($\Delta t = T_{\text{key}[n]} - T_{\text{key}[n-1]}$) được lọc mượt qua EMA. |
| **$t$** | Hệ số tốc độ ngón tay (Trọng số nội suy) | *Không đơn vị (vô thứ nguyên)* | `double` ($0.0 \le t \le 1.0$) | Tỷ lệ phần trăm thể hiện mức độ chậm rãi của ngón tay: $t = 0.0$ ($0\%$, gõ siêu nhanh) $\to$ $t = 1.0$ ($100\%$, gõ thong thả). |
| **$\text{base\_us}$** | Độ trễ nền cơ sở trước khi Commit | Microsecond ($\mu\text{s}$) | `double` / `uint64_t` | Khoảng thời gian nghỉ tĩnh sau khi nuốt phím để ổn định con trỏ văn bản trước khi gọi `ic_->commitString()`. |
| **$\text{min\_per\_bs\_us}$** | Sàn thời gian xóa mỗi phím theo ngón tay | Microsecond ($\mu\text{s}$) | `double` | Mức thời gian tối thiểu phân bổ cho mỗi phím xóa tính theo tốc độ ngón tay người dùng. |
| **$\text{per\_bs\_us}$** | Thời gian tiêu thụ thực tế cho mỗi phím | Microsecond ($\mu\text{s}$) | `double` / `uint64_t` | Thời gian xóa mỗi phím sau khi kết hợp cả 3 lớp bảo vệ: Áp lực tay gõ, Lịch sử EMA, và Phản xạ đo tức thì. |
| **$T_{\text{tổng}}$ (`micro_delay_us`)** | Tổng vi trễ chờ commit string | Microsecond ($\mu\text{s}$) | `uint64_t` | Tổng thời gian Fcitx5 hẹn giờ (`EventLoop::addTimeEvent`) chờ trước khi bắn ký tự tiếng Việt mới vào ứng dụng. |
| **$\text{WatchdogCeiling}$** | Trần an toàn cắt lỗ khẩn cấp | Millisecond ($\text{ms}$) | `uint64_t` | Mức trần tối đa cho phép chờ: Chuẩn hóa **$250\text{ms}$** toàn hệ thống. Nếu App lag quá 250ms, Fcitx5 kích hoạt cắt lỗ khẩn cấp bảo vệ bàn phím. |

---

## ⚙️ 2. QUY TRÌNH 4 BƯỚC TÍNH TOÁN CHI TIẾT

```text
  [Bước 0: Bấm giờ Vòng lặp Uinput] ──► Đo T_measured & Cập nhật EMA app_ack
                 │
  [Bước 1: Chuẩn hóa IKI ngón tay]  ──► Ra tỷ lệ phần trăm t ∈ [0.0, 1.0]
                 │
  [Bước 2: Tính độ trễ nền base_us] ──► base_us = lerp(1ms, 15ms, t)
                 │
  [Bước 3: Lớp bảo vệ 3 tầng per_bs]──► per_bs_us = max(min_per_bs, app_ack, T_measured)
                 │
  [Bước 4: Tổng hợp T_tổng]         ──► T_tổng = base_us + N * per_bs_us
```

---

### 🔹 BƯỚC 0: Đo Đạc Thời Gian Nuốt Phím Xóa Thực Tế ($\Delta T_{\text{swallow}}$)

1. **Bấm giờ tại nguồn phát:**
   Trong `LilypadState::performReplacement()`, ngay trước khi ghi mảng sự kiện vào `/dev/uinput`:
   $$T_1 = \text{steady\_clock::now()}$$

2. **Chốt giờ tại đích đến:**
   Trong `LilypadState::handleUInputKeyPress()`, ngay khi phím Sentinel thứ $N+1$ quay về Fcitx5:
   $$T_2 = \text{steady\_clock::now()}$$
   $$\Delta T_{\text{swallow}} = T_2 - T_1 \quad [\mu\text{s}]$$

3. **Tính nhịp nuốt thô tức thì của mỗi phím:**
   - **Với $N = 1$ (thao tác đổi dấu / nguyên âm đơn):** Toàn bộ vòng lặp $\Delta T_{\text{swallow}}$ phản ánh trực tiếp thời gian hệ thống và ứng dụng xử lý thao tác xóa đó (không chia nhỏ làm méo mó độ trễ):
     $$T_{\text{measured}} = \Delta T_{\text{swallow}} \quad [\mu\text{s}]$$
   - **Với $N \ge 2$ (thay thế chuỗi vần dài):**
     $$T_{\text{measured}} = \max\left(\frac{\Delta T_{\text{swallow}}}{N + 1}, \; \frac{\Delta T_{\text{swallow}}}{2}\right) \quad [\mu\text{s}]$$

4. **Cập nhật bộ lọc làm mịn EMA (Exponential Moving Average):**
   $$\text{app\_ack}_{\text{new}} = 0.35 \times T_{\text{measured}} + 0.65 \times \text{app\_ack}_{\text{prev}} \quad [\mu\text{s}]$$
   *Giới hạn an toàn:* $\text{app\_ack} \in [1.000\,\mu\text{s}, \; 250.000\,\mu\text{s}]$.

---

### 🔹 BƯỚC 1: Chuẩn Hóa Nhịp Ngón Tay IKI Về Hệ Số Phần Trăm $t$

Nhịp gõ ngón tay $\text{IKI}$ (tính bằng mili-giây) được co giãn về đoạn $[0.0, 1.0]$ qua hàm Min-Max Feature Scaling:

$$t = \text{clamp}\left(\frac{\text{IKI} - 35}{150 - 35}, \; 0.0, \; 1.0\right) \quad [\text{Không đơn vị}]$$

* **Trường hợp $\text{IKI} \le 35\text{ms}$ (Gõ lướt cực hạn - Burst Typing):** $t = 0.0$ ($0\%$).
* **Trường hợp $\text{IKI} \ge 150\text{ms}$ (Gõ chậm / thong thả):** $t = 1.0$ ($100\%$).
* **Trường hợp $35\text{ms} < \text{IKI} < 150\text{ms}$:** $t$ là số thực liên tục đại diện cho tỷ lệ phần trăm tốc độ (ví dụ $\text{IKI} = 92.5\text{ms} \implies t = 0.50$).

> [!NOTE]
> Riêng từ đầu tiên khi vừa chuyển cửa sổ hoặc chưa có dữ liệu gõ ($\text{IKI} == 0$), áp dụng Cold Start Safe Baseline: $35.000\,\mu\text{s} + N \times 15.000\,\mu\text{s}$ để bảo đảm $100\%$ an toàn.

---

### 🔹 BƯỚC 2: Tính Độ Trễ Nền Cơ Sở ($\text{base\_us}$)

Độ trễ nền được nội suy tuyến tính (`lerp`) từ mức sàn siêu tốc $1.000\,\mu\text{s}$ ($1\text{ms}$) đến mức an toàn $15.000\,\mu\text{s}$ ($15\text{ms}$):

$$\text{base\_us} = \text{lerp}(1.000\,\mu\text{s}, \; 15.000\,\mu\text{s}, \; t) = 1.000 + t \times (15.000 - 1.000) = 1.000 + t \times 14.000 \quad [\mu\text{s}]$$

* Khi $t = 0.0$ (gõ siêu nhanh): $\text{base\_us} = \mathbf{1.000\,\mu\text{s}}$ ($1.0\text{ms}$).
* Khi $t = 0.5$ (gõ trung bình): $\text{base\_us} = \mathbf{8.000\,\mu\text{s}}$ ($8.0\text{ms}$).
* Khi $t = 1.0$ (gõ thong thả): $\text{base\_us} = \mathbf{15.000\,\mu\text{s}}$ ($15.0\text{ms}$).

---

### 🔹 BƯỚC 3: Tính Thời Gian Tiêu Thụ Mỗi Phím Xóa Theo Lớp Bảo Vệ 3 Tầng ($\text{per\_bs\_us}$)

#### 1. Tính mức sàn theo ngón tay (`min_per_bs_us`):
$$\text{min\_per\_bs\_us} = \text{lerp}(500\,\mu\text{s}, \; 18.000\,\mu\text{s}, \; t) = 500 + t \times (18.000 - 500) = 500 + t \times 17.500 \quad [\mu\text{s}]$$

#### 2. Cơ chế Bảo vệ 3 Tầng (Tri-Layer Safety Protection):
$$\mathbf{\text{per\_bs\_us}} = \max\Big(\underbrace{\text{min\_per\_bs\_us}}_{\text{Lớp 1: Sàn ngón tay } t}, \quad \underbrace{\text{app\_ack}_{\text{new}}}_{\text{Lớp 2: EMA mượt mà}}, \quad \underbrace{T_{\text{measured}}}_{\text{Lớp 3: Phản xạ tức thì}}\Big) \quad [\mu\text{s}]$$

* **Ý nghĩa Lớp 1 (`min_per_bs_us`):** Tôn trọng tốc độ ngón tay của người dùng khi máy cực kỳ mượt.
* **Ý nghĩa Lớp 2 (`app_ack_new`):** Duy trì nhịp độ ổn định, không làm bàn phím bị giật cục/khựng bất ngờ.
* **Ý nghĩa Lớp 3 ($T_{\text{measured}}$):** Kích hoạt cấp cứu ngay lập tức trong $1\,\mu\text{s}$ nếu App vừa đột ngột bị nghẽn (spike/lag) ở chính lần gõ này, không để rơi rụng bất kỳ ký tự nào.

---

### 🔹 BƯỚC 4: Tổng Hợp Thời Gian Vi Trễ Hoàn Chỉnh ($T_{\text{tổng}}$)

Tổng thời gian vi trễ trước khi gọi `ic_->commitString()`:

$$\mathbf{T_{\text{tổng}}} = \text{base\_us} + N \times \mathbf{\text{per\_bs\_us}} \quad [\mu\text{s}]$$

#### Giới hạn Sàn và Trần An toàn (Boundary Clamping):
$$T_{\text{tổng}} = \text{clamp}\Big(T_{\text{tổng}}, \; \text{Floor}_{\text{App}}, \; 250.000\,\mu\text{s}\Big)$$

* **Trần an toàn chuẩn hóa:** **$250\text{ms}$** cho toàn bộ hệ thống Linux.
  - Trên môi trường Linux Wayland asynchronous IPC, $250\text{ms}$ là giới hạn vật lý tiêu chuẩn cho độ nhạy giao diện (UI responsiveness).
  - Khi đã có Cảm biến Nuốt phím ACK đo đạc thời gian thực, nếu một ứng dụng vượt quá $250\text{ms}$ thì đó là tình trạng đóng băng thực sự (True Freeze), cơ chế cắt lỗ khẩn cấp $250\text{ms}$ kích hoạt để giải phóng bàn phím.
* **Sàn an toàn theo Ứng dụng:**
  - Ứng dụng thông thường (Terminal, Chrome, IDE): $\text{Floor} = 1.000\,\mu\text{s}$ ($1\text{ms}$).
  - Antigravity 2.0 (Lexical Editor AI plugins): $\text{Floor} = 32.000\,\mu\text{s}$ ($32\text{ms}$).

---

## 📊 3. BẢNG ĐỐI CHỨNG ĐỊNH LƯỢNG TRÊN CÁC KỊCH BẢN THỰC TẾ

| Kịch bản ứng dụng | Số phím xóa ($N$) | Nhịp ngón tay ($\text{IKI}$) | $\Delta T_{\text{swallow}}$ nuốt phím | $T_{\text{measured}}$ mỗi phím | Lớp quyết định ở Bước 3 | $T_{\text{tổng}}$ tính ra | Trải nghiệm gõ thực tế |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Terminal (Ghostty / Kitty)** | $N=1$ (`a` $\to$ `á`) | $25\text{ms}$ ($t=0.0$) | $2.0\text{ms}$ | $2.0\text{ms}$ | Lớp 3 ($T_{\text{mea}} = 2.0\text{ms}$) | $1.0\text{ms} + 1 \times 2.0\text{ms} = \mathbf{3.0\text{ms}}$ | **Zero-Latency tức thì 100%**, cực kỳ nhạy |
| **Chrome bình thường** | $N=2$ (`châu` $\to$ `cháu`) | $92.5\text{ms}$ ($t=0.5$) | $12.0\text{ms}$ | $6.0\text{ms}$ | Lớp 1 (`min_per_bs` $= 9.25\text{ms}$) | $8.0\text{ms} + 2 \times 9.25\text{ms} = \mathbf{26.5\text{ms}}$ | Chữ lên mượt mà, con trỏ không giật |
| **Antigravity (Chat mới, nhẹ)** | $N=1$ (`nhìn` $\to$ `nhn`) | $80\text{ms}$ ($t=0.39$) | $10.0\text{ms}$ | $10.0\text{ms}$ | Kẹp sàn Antigravity $32\text{ms}$ | $\max(16.5\text{ms}, 32\text{ms}) = \mathbf{32.0\text{ms}}$ | Phản hồi nhanh, an toàn cho Lexical Editor |
| **Antigravity (DOM nặng $134\text{k}$ nodes)** | $N=1$ (`qu` $\to$ `quá`) | $90\text{ms}$ ($t=0.48$) | **$70.0\text{ms}$** | **$70.0\text{ms}$** | **Lớp 3 ($T_{\text{measured}} = 70\text{ms}$)** | $7.7\text{ms} + 1 \times 70\text{ms} = \mathbf{77.7\text{ms}}$ | **Tự động dãn delay cứu chữ, KHÔNG RỤNG `qu`!** |
| **App bị đơ cứng (Frozen)** | Bất kỳ | Bất kỳ | Quá $250\text{ms}$ không có Sentinel | N/A | Watchdog Hard Timeout ($250\text{ms}$) | Kích hoạt `purgeContextEmergency()` | Bàn phím không bao giờ bị treo, xả phím thô an toàn |

---

## 💡 4. TÓM TẮT ĐIỂM SÁNG KIẾN TRÚC

1. **Tính Tự Thích Ứng Kép (Dual-Loop Control):** Vừa lắng nghe tốc độ người dùng gõ qua $\text{IKI}$ ($t$), vừa đo lường sức chịu tải thực tế của hệ thống qua $\Delta T_{\text{swallow}}$.
2. **Khắc phục Triệt để Điểm Mù Cũ:** Không còn phụ thuộc mù quáng vào các hằng số gán chết, giúp bộ gõ nhận diện được khi nào Antigravity bị nghẽn để dãn nhịp an toàn.
3. **Bảo tồn Tuyệt đối Tốc độ Ứng Dụng Khác:** Mọi cải tiến cho Antigravity chỉ kích hoạt thông qua cơ chế phản xạ đo đạc thực tế; Terminal và Chrome nhẹ tiếp tục hoạt động ở tốc độ bàn thờ $1.0\text{ms} \sim 2.5\text{ms}$.
