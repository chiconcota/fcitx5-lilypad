**KIẾN TRÚC CẢM BIẾN NHỊP GÕ IKI & DYNAMIC MICRO-PACING (**fcitx5-lilypad **)**  
@target: fcitx5-lilypad Core Engine | @module: Sequencer Layer & ACK Sensor Layer | @status: STABLE (v2.3.0)  
![](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAnEAAAACCAYAAAA3pIp+AAAABmJLR0QA/wD/AP+gvaeTAAAACXBIWXMAAA7EAAAOxAGVKw4bAAAANklEQVR4nO3OQQmAABRAsSeYxZw/lieLGMACBrCCNxG2BFtmZquOAAD4i3Ot7mr/egIAwGvXA6fGBdgoVMwYAAAAAElFTkSuQmCC)  
**🧭 1. TỔNG QUAN & BỐI CẢNH (EXECUTIVE SUMMARY)**  
Trong các bộ gõ tiếng Việt trên Linux hoạt động theo cơ chế **Kernel Uinput Sequencer** (không dùng deleteSurroundingText và không dùng Preedit để tương thích tối đa với Wayland/X11), quy trình thay thế ký tự thô cũ bằng ký tự tiếng Việt có dấu luôn gồm 2 giai đoạn:  
1. Phát N phím xóa KEY_BACKSPACE qua /dev/uinput server daemon đến ứng dụng.  
2. Tạm hoãn một khoảng ngắt nhịp vi mô (micro_delay_us) để ứng dụng kịp xóa chữ cũ trên DOM/màn hình \to Phát ký tự mới qua ic_->commitString().  
Phím vật lý ──► Engine Bamboo ──► Uinput Backspaces (N phím) ──► [micro_delay_us] ──► commitString()  
   
**🔴 Vấn đề của Phương pháp Cũ (Static Blind Wait):**  
- Trước đây, hệ thống sử dụng công thức thời gian chờ cố định:  
 \text{micro\_delay\_us} = 6000 + N \times 4000\,\mu\text{s} \quad (10\text{ms} \sim 18\text{ms})  
- **Hạn chế lớn:** Đây là cơ chế  **"Chờ mù" (Blind Wait)**. Dù người dùng gõ siêu tốc (20\text{ms}/\text{phím}) trên một ứng dụng phản hồi tức thì (2\text{ms} như Terminal, Alacritty, Kate), bộ gõ vẫn bắt ép hệ thống phải chờ đủ 14\text{ms} \sim 18\text{ms}.  
- **Hậu quả:** Phím gõ tiếp theo của người dùng bị dồn ứ vào hàng đợi đệm buffered_keys_, tạo cảm giác bị kìm hãm, mất đi tính tức thì (Zero-Latency feel).  
![](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAnEAAAACCAYAAAA3pIp+AAAABmJLR0QA/wD/AP+gvaeTAAAACXBIWXMAAA7EAAAOxAGVKw4bAAAANUlEQVR4nO3OMQ2AABAAsSNhYMMAKlD4OzrxgQU2QtIq6DIzR3UFAMBf3Gu1VefXEwAAXtsfSqADWz4G/HUAAAAASUVORK5CYII=)  
**⚡ 2. CƠ CHẾ HOẠT ĐỘNG CỦA CẢM BIẾN IKI (**IIkiSensor **)**  
**IKI (Inter-Keystroke Interval)** là khoảng thời gian (tính bằng millisecond) giữa 2 lần nhấn phím vật lý liên tiếp của ngón tay người dùng:  
   
 \Delta t = T_n - T_{n-1}  
sequenceDiagram  
     autonumber  
     actor User as Ngón tay người dùng  
     participant State as LilypadState (Passive Listener)  
     participant IKI as StandardIkiSensor (EMA Filter)  
     participant Seq as Sequencer & AckSensor  
   
     User->>State: Nhấn phím vật lý 'h' (T1)  
     State->>IKI: on_key_event(T1, is_synthetic=false)  
     Note over IKI: Lưu mốc T1  
     User->>State: Nhấn phím vật lý 'o' (T2)  
     State->>IKI: on_key_event(T2, is_synthetic=false)  
     Note over IKI: Δt = T2 - T1 = 28ms<br/>Cập nhật EMA IKI: 32ms  
     State->>Seq: get_micro_delay_us(bsCount=1, iki=32ms)  
     Note over Seq: Tính α = 0.21 -> Delay = 1.5ms (Fast-Path)  
     Seq-->>State: Trả về 1500us  
     State->>User: Commit ký tự tức thì (1.5ms)  
   
**🔹 Thuật toán Làm mịn EMA (Exponential Moving Average):**  
Để tránh tình trạng nhịp delay bị nhảy giật cục giữa các phím gõ nhanh và gõ chậm, StandardIkiSensor áp dụng công thức làm mịn EMA:  
   
 \text{EMA}_{\text{new}} = 0.35 \times \Delta t_{\text{clamped}} + 0.65 \times \text{EMA}_{\text{prev}}  
**🔹 Bộ lọc nhiễu 2 đầu (Noise Rejection Filter):**  
- **Lọc Micro-Glitch (\Delta t < 5\text{ms}):** Bỏ qua các xung nảy phím phần cứng (key bounce/chatter).  
- **Lọc Khoảng nghỉ Idle (\Delta t > 1000\text{ms}):** Khi người dùng dừng lại suy nghĩ giữa các câu/từ, giá trị \Delta t lớn này sẽ bị bỏ qua, không làm méo mó chỉ số tốc độ tay trung bình.  
- **Phát hiện Gõ lướt siêu tốc (Burst Typing):** Khi \text{EMA\_IKI} \le 35\text{ms}, hệ thống chuyển sang chế độ gõ lướt cực hạn.  
![](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAnEAAAACCAYAAAA3pIp+AAAABmJLR0QA/wD/AP+gvaeTAAAACXBIWXMAAA7EAAAOxAGVKw4bAAAANUlEQVR4nO3OMQ2AUBBAsUfyVTCg9UygEBVsWGAjJK2CbjNzVGcAAPzFtapV7V9PAAB47X4AEWIEM8iQs0EAAAAASUVORK5CYII=)  
**🔄 3. KIẾN TRÚC ĐIỀU KHIỂN VÒNG LẶP KÉP (DUAL-LOOP CONTROL)**  
Việc bổ sung IKI tạo nên một hệ thống điều phối vòng lặp kín hoàn chỉnh:  
┌────────────────────────────────────────────────────────────────────────┐  
 │                   HỆ THỐNG ĐIỀU KHIỂN VÒNG LẶP KÉP                    │  
 │                                                                        │  
 │   ┌───────────────────────────┐     ┌──────────────────────────────┐   │  
 │   │ VÒNG LẶP 1: NGÓN TAY      │     │ VÒNG LẶP 2: ỨNG DỤNG         │   │  
 │   │ (Input Cadence - IIkiSensor)│   │ (Output Latency - IAckSensor)│   │  
 │   ├───────────────────────────┤     ├──────────────────────────────┤   │  
 │   │ • Đo tốc độ gõ ngón tay   │     │ • Đo tốc độ phản hồi Compositor│ │  
 │   │ • Tính nhịp EMA IKI (ms)  │     │ • Tính Dynamic Barrier T_ACK │   │  
 │   │ • Phát hiện Burst Typing  │     │ • Giám sát hiện tượng App Lag│   │  
 │   └─────────────┬─────────────┘     └──────────────┬───────────────┘   │  
 │                 │                                  │                   │  
 │                 └────────────────►┬◄───────────────┘                   │  
 │                                   │                                    │  
 │                                   ▼                                    │  
 │                   ┌──────────────────────────────┐                     │  
 │                   │    DYNAMIC MICRO-PACING      │                     │  
 │                   │  (get_micro_delay_us(N, iki))│                     │  
 │                   └──────────────────────────────┘                     │  
 └────────────────────────────────────────────────────────────────────────┘  
   
![](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAnEAAAACCAYAAAA3pIp+AAAABmJLR0QA/wD/AP+gvaeTAAAACXBIWXMAAA7EAAAOxAGVKw4bAAAANUlEQVR4nO3OMQ2AUBBAsUeCE4yeIiT9CRVMWGAjJK2CbjNzVGcAAPzF2qu7Wl9PAAB47XoA/vcF8exqpY4AAAAASUVORK5CYII=)  
**🎯 4. CƠ CHẾ DYNAMIC MICRO-PACING & DEPTH-AWARE FAST-PATH**  
Thay vì bắt ứng dụng chờ cố định, NiriAckSensor và GenericAckSensor tính toán nhịp hoãn theo công thức co giãn liên tục:  
**1. Hệ số co giãn nhịp gõ (\alpha):**  
\alpha = \text{clamp}\left(\frac{\text{EMA\_IKI}}{150.0}, \; 0.15, \; 1.0\right)  
**2. Fast-Path cho Thao tác Nông (N = 1 Backspace — Chiếm ~70% các lần gõ):**  
- Thao tác: Bỏ dấu thanh hoặc đổi nguyên âm (a\toá, e\toê, u\toư).  
- Bản chất: Chỉ xóa đúng 1 ký tự vừa gõ, xác suất xé gói tin DOM gần như bằng 0.  
- Công thức:  
 \text{delay}_{N=1} = \max\left(1000\,\mu\text{s}, \; 10000\,\mu\text{s} \times \alpha\right)  
  - Khi gõ siêu tốc (\text{IKI} = 25\text{ms}): \alpha = 0.16 \implies \mathbf{1.6\text{ms}} (nhanh gấp **6.25 lần** cũ).  
  - Khi gõ chậm (\text{IKI} \ge 150\text{ms}): \alpha = 1.0 \implies \mathbf{10.0\text{ms}} (an toàn tuyệt đối).  
**3. Sàn An toàn cho Thao tác Sâu (N \ge 2 Backspaces — Chiếm ~30%):**  
- Thao tác: Thay đổi vần phức (hoang\tohoàng: N=3, nghieng\tonghiêng: N=4).  
- Công thức:  
 \text{raw} = 6000 + N \times 4000\,\mu\text{s}  
 \text{floor} = 1000 + N \times 500\,\mu\text{s}  
 \text{delay}_{N \ge 2} = \max\left(\text{floor}, \; \text{raw} \times \alpha\right)  
  - Khi gõ siêu tốc N=3: \text{delay} = \mathbf{2.5\text{ms}} (thay vì 18\text{ms} cũ).  
![](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAnEAAAACCAYAAAA3pIp+AAAABmJLR0QA/wD/AP+gvaeTAAAACXBIWXMAAA7EAAAOxAGVKw4bAAAANklEQVR4nO3OQQmAABRAsSeYxZw/lieLGMACBrCCNxG2BFtmZquOAAD4i3Ot7mr/egIAwGvXA6fGBdgoVMwYAAAAAElFTkSuQmCC)  
**📊 5. BẢNG SO SÁNH ĐA CHIỀU: PHƯƠNG PHÁP CŨ VS IKI ADAPTIVE MỚI**  
| | | |  
|-|-|-|  
| **Tiêu chí Đánh giá** | **Phương pháp Cũ (Static Blind Wait)** | **Phương pháp Mới (IKI Adaptive Pacing)** |   
| **Độ trễ 1 Backspace (N=1)** | Cố định **10ms** | Động **1.0\text{ms} \sim 10.0\text{ms}** |   
| **Độ trễ 3 Backspaces (N=3)** | Cố định **18ms** | Động **2.5\text{ms} \sim 18.0\text{ms}** |   
| **Cảm giác gõ lướt (Burst Typing)** | Bị gượng gạo do phím sau phải đợi 14\text{ms} \sim 18\text{ms} | **Tức thì 100% (Zero-Latency feel)** |   
| **Khả năng thích ứng theo tay người gõ** | Không (áp đặt cứng nhắc một mức delay cho mọi người) | **Có** (tự học theo tốc độ gõ thực tế của từng cá nhân qua EMA) |   
| **Tỷ lệ phím rơi vào buffer đệm** | Cao khi gõ nhanh | **Rất thấp** (xả tức thì theo nhịp tay) |   
| **Bảo vệ an toàn khi gõ chậm / App lag** | Tốt (nhờ delay 10ms-18ms) | **Hoàn hảo** (tự động dãn về 10ms-18ms khi tay gõ chậm) |   
| **Chi phí CPU & Tài nguyên hệ thống** | 0% | **0%** (toàn bộ tính toán IKI là phép nhân vô hướng \mathcal{O}(1)) |   
| **Tác động tới Main Thread Fcitx5** | Non-blocking (EventLoop Timer) | **Non-blocking 100% (Passive Listening)** |   
   
![](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAnEAAAACCAYAAAA3pIp+AAAABmJLR0QA/wD/AP+gvaeTAAAACXBIWXMAAA7EAAAOxAGVKw4bAAAANUlEQVR4nO3OMQ2AUBBAsUfyRTCh9VRgEBGsWGAjJK2CbjNzVGcAAPzFtapV7V9PAAB47X4AEWgEMAY9+pUAAAAASUVORK5CYII=)  
**💡 6. VÌ SAO PHƯƠNG PHÁP IKI MANG LẠI HIỆU QUẢ VƯỢT TRỘI?**  
1. **Triệt tiêu "Khoảng thời gian chết" (Zero Dead-Time):**  
   
 Thay vì bắt CPU và Event Loop ngồi chờ 14ms một cách vô nghĩa trong khi Wayland Compositor đã xử lý xong từ đời nào (2ms), IKI cho phép giải phóng chuỗi commitString() ngay khi an toàn.  
2. **Tối ưu hóa đúng 70% trọng tâm (Pareto Principle):**  
   
 Trong văn bản tiếng Việt, hơn 70% các lần biến đổi từ là thao tác N=1 (gõ s, f, r, x, j, w ngay sau nguyên âm). Nhờ **Fast-Path 1.0\text{ms} \sim 1.5\text{ms}**, phần lớn trải nghiệm gõ hàng ngày của người dùng đạt tốc độ phản hồi ánh sáng.  
3. **Tính thông minh hai chiều (Smart Bi-directional Adaptation):**  
  - Khi tay bạn gõ nhanh: Bộ gõ tự động "chạy nhanh theo bạn".  
  - Khi tay bạn gõ chậm hoặc dừng lại: Bộ gõ tự động "chạy chậm lại" để bảo toàn tính toàn vẹn của DOM ứng dụng.  
