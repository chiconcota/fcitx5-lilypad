# VNLILYPAD LOTUS DECISION LOG (NHẬT KÝ QUYẾT ĐỊNH KIẾN TRÚC)

> **Kiến trúc Chuẩn:** Fcitx5 Lilypad Sequencer Architecture (`v2.2.0-modular-sensor`)
> **Nguyên tắc Đối soát:** Chỉ lưu trữ các Quyết định Kỹ thuật đang thực tế vận hành 100% trong mã nguồn C++ của `fcitx5-lilypad`.
> **Lưu trữ thử nghiệm đã gỡ bỏ [XOÁ - KHÔNG ÁP DỤNG]:** Xem chi tiết tại [archive/deprecated-decisions.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.vnlilypadlotus-ai/2-memory/archive/deprecated-decisions.md).

---

## 🎯 1. HỆ THỐNG CẢM BIẾN ADAPTIVE ACK, IKI & DYNAMIC LATENCY CONTROL

### [2026-08-25] Quyết định 024: Cold Start Safe Baseline ($>50\text{ms}$ cho chữ đầu tiên)
- **Bối cảnh:** Khi khởi động hoặc vừa chuyển focus sang cửa sổ mới, hệ thống chưa có dữ liệu quá khứ ($\text{IKI} = 0$, $\text{ACK}$ chưa đo). Nếu dùng giá trị giả định quá nhanh, chữ đầu tiên có nguy cơ bị lỗi trên các trình soạn thảo web nặng.
- **Quyết định:**
  1. **Thiết lập mức trần an toàn tuyệt đối cho chữ đầu tiên (`iki_ms == 0`):**
     $$\Delta t(N) = 35\text{ms} + N \times 15\text{ms}$$
     - $N=1 \implies \mathbf{50\text{ms}}$
     - $N=2 \implies \mathbf{65\text{ms}}$
     - $N=3 \implies \mathbf{80\text{ms}}$
  2. **Chuyển giao thích ứng tức thì:** Ngay từ chữ thứ 2 trở đi khi $\text{IKI} > 0$, thuật toán Lerp và App ACK tự động tiếp quản và co giãn theo tốc độ thực tế (xuống tới $2.5\text{ms}$ trên Terminal).
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/ack-sensors/niri-sensor.h`, `generic-sensor.h`.

### [2026-08-25] Quyết định 023: App ACK Backspace Consumption Dynamic Integration ($N \times T_{\text{ack}}$)
- **Bối cảnh:** Trong các công thức trước, thời gian vi trễ $\Delta t$ chỉ phụ thuộc vào nhịp tay $\text{IKI}$ và số phím $N$ với hệ số hằng số, thiếu mất biến số thời gian App tiêu thụ xong $N$ phím xóa trong JavaScript/DOM Engine. Điều này khiến Facebook/Chrome (vốn cần $>45\text{ms} \sim 50\text{ms}$ cho $N \ge 2$) bị thiếu thời gian và nuốt chữ.
- **Quyết định:**
  1. **Tích hợp biến số thời gian tiêu thụ App ACK:**
     $$T_{\text{per\_bs}} = \max\Big(\text{lerp}(500\mu\text{s}, 18000\mu\text{s}, t), \; T_{\text{app\_ack}}\Big)$$
     $$\Delta t(N) = \text{lerp}(1000\mu\text{s}, 15000\mu\text{s}, t) + N \cdot T_{\text{per\_bs}}$$
  2. **Hiệu quả thực tế:**
     - Trên Terminal / App nhẹ: $T_{\text{app\_ack}} \approx 1\text{ms}, N=2 \implies \mathbf{3.0\text{ms}}$ (Zero-Latency tức thì).
     - Trên Facebook / Web DOM: $T_{\text{app\_ack}} \approx 18\text{ms} \sim 20\text{ms}, N=2 \implies \mathbf{51.0\text{ms}}$ (Bảo đảm React DOM tiêu thụ hết phím xóa trước khi commit `"áu"`).
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/ack-sensors/niri-sensor.h`, `generic-sensor.h`.

### [2026-08-25] Quyết định 022: Uinput N+1 Sentinel Barrier Protocol (Triệt tiêu Race Condition Backspace xóa nhầm Commit)
- **Bối cảnh:** Khi xóa $N$ ký tự cũ và commit chuỗi mới, nếu chỉ phát đúng $N$ phím Backspace từ uinput thì phím thứ $N$ khi tới Fcitx5 sẽ được chuyển tiếp xuống App cùng lúc với lệnh `commitString`. Trên các app phức tạp (Chrome/React DOM), lệnh Commit chen lên trước và phím Backspace thứ $N$ nổ sau xóa mất chuỗi vừa commit (`cháu` thành `ch`).
- **Quyết định:**
  1. **Phát $N + 1$ phím Backspace qua `/dev/uinput`:**
     - $N$ phím đầu được chuyển tiếp nguyên vẹn (`return false;`) xuống App để xóa sạch $N$ ký tự cũ.
     - Phím thứ $N + 1$ đóng vai trò **Phím Rào Chắn Sentinel**: Fcitx5 nuốt trọn phím này (`event.filterAndAccept(); return true;`) và không cho gửi xuống App.
  2. **Bảo đảm trật tự vật lý FIFO:** Sự xuất hiện của phím $N+1$ tại Fcitx5 là bằng chứng phần cứng chứng minh $N$ phím trước đã đi vào App an toàn. Khi commit nổ, App đã xóa xong và không còn bất kỳ phím Backspace nào bám sau để xóa nhầm ký tự mới.
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/lilypad-state.cpp:L452-L488, L542-L585`.

### [2026-08-25] Quyết định 021: Normalized Linear Interpolation (Lerp) for Micro-Pacing & Uniform Web IME Routing
- **Bối cảnh:** Việc quy định cứng các con số thời gian gây ra hai hệ quả tiêu cực: hoặc bóp nghẹt các ứng dụng cực nhẹ (như Terminal/Alacritty khi gõ nhanh), hoặc làm lệch nhịp DOM reconciler trên các trình soạn thảo web phức tạp (React/Draft.js trên Facebook).
- **Quyết định:**
  1. **Áp dụng Mô hình Chuẩn hóa Min-Max & Nội suy Tuyến tính (Lerp):**
     $$t = \text{clamp}\left(\frac{\text{EMA\_IKI} - 35}{150 - 35}, \; 0.0, \; 1.0\right)$$
     $$\Delta t(N) = \text{lerp}(1000\mu\text{s}, 6000\mu\text{s}, t) + N \cdot \text{lerp}(500\mu\text{s}, 4000\mu\text{s}, t)$$
     - Khi gõ siêu tốc (Burst Typing $\le 35\text{ms}$): $\Delta t$ nén xuống sàn vật lý **$1.5\text{ms} \sim 2.5\text{ms}$** (Zero-Latency tức thì cho Terminal).
     - Khi gõ bình thường: $\Delta t$ điều hòa mượt mà theo nhịp ngón tay ($5\text{ms} \sim 10\text{ms}$).
     - Khi gõ thong thả: $\Delta t$ đạt mức an toàn ($13\text{ms} \sim 18\text{ms}$).
  2. **Đồng bộ hóa kênh phát cho Chromium (`wa_chromium_flag == true`):** Mọi ký tự gõ thường, chuỗi xóa vi mô và phím chốt Space đều được phát đồng nhất qua `ic_->commitString()`, ngăn ngừa xung đột Virtual DOM trên Facebook / Google Docs.
  3. **Bảo tồn kênh Native Key cho GTK4 (`wa_chromium_flag == false`):** Phím Space tiếp tục dùng `keyEvent.forward()` kết hợp `accuracy = 1µs`, triệt tiêu 100% hiện tượng đảo dấu cách trên Gnome Text Editor.
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/ack-sensors/niri-sensor.h`, `generic-sensor.h`, `lilypad-engine.cpp:L429`, `lilypad-state.cpp:L718-L735`.

### [2026-08-25] Quyết định 020: High-Precision Timer Accuracy (`accuracy = 1µs`)
- **Bối cảnh:** Tham số `accuracy = 0` trong `EventLoop::addTimeEvent` bị systemd `sd-event` mặc định áp mức trễ 250ms (coalescing), khiến `commit_timer_` bị dồn cục không nổ đúng lúc và va chạm với phím Space mới (`"c òngi"`, `"l àcười"`).
- **Quyết định:** Đặt tham số `accuracy = 1` ($1\,\mu\text{s}$) cho `commit_timer_` và replay timers, buộc Kernel Linux phải đánh thức Event Loop chính xác tức thì ở mốc micro-delay mà không bị hoãn lười 250ms.
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/lilypad-state.cpp:L465,L479,L573`.

### [2026-08-25] Quyết định 019: Two-Tier Timeout (Dynamic Soft Timeout & Watchdog Hard Timeout 250ms) & State Protection (Phase 4.3)
- **Bối cảnh:** Khi ứng dụng bị nghẽn (DOM render lag, Garbage Collection stall) hoặc đóng băng, các phím gõ tiếp theo có thể bị rơi vào tình trạng xung đột hoặc kẹt bàn phím.
- **Quyết định:**
  1. **Dynamic Soft Timeout ($T_{\text{soft}}$):** Kết hợp nhịp ngón tay $\text{EMA\_IKI}$ và độ trễ phản hồi $T_{\text{expected}}$ của App qua công thức $T_{\text{soft}} = \text{clamp}(\max(T_{\text{expected}} \times 2.0, \min(\text{IKI}, T_{\text{expected}} + 30)), 35\text{ms}, 120\text{ms})$. Khi $T_{\text{elapsed}} \ge T_{\text{soft}}$, Sequencer chuyển sang `BarrierState::AppLagHolding`, tạm hoãn phát uinput tiếp theo và gom phím an toàn vào RAM `buffered_keys_` để chống rách chữ.
  2. **Watchdog Hard Timeout (250ms):** Main Event Loop cài đặt timer 250ms độc lập mỗi khi bắt đầu `performReplacement()`. Tự động hủy khi commit thành công.
  3. **Cắt lỗ Khẩn cấp (`purgeContextEmergency`):** Nếu App bị treo cứng quá 250ms, hệ thống lập tức reset Bamboo Engine, xóa word buffer và xả toàn bộ phím đệm trong RAM ra màn hình dưới dạng phím thô (`ic_->forwardKey()`), bảo đảm bàn phím không bao giờ bị đơ hay mất ký tự.
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/lilypad-sequencer.h/.cpp`, `fcitx5-lilypad/src/lilypad-state.h/.cpp`.

### [2026-08-25] Quyết định 018: Dynamic Micro-Pacing Optimization & Depth-Aware Fast-Path (Phase 4.2)
- **Bối cảnh:** Trước đây `micro_delay_us` bị cố định mù ($6\text{ms} + N \times 4\text{ms} = 10\text{ms} \sim 18\text{ms}$) sau khi bắn phím Backspace, gây dồn ứ phím vào hàng đợi `buffered_keys_` không cần thiết khi người dùng gõ siêu tốc trên app mượt.
- **Quyết định:**
  1. Cập nhật `IAckSensor::get_micro_delay_us(int bsCount, uint64_t iki_ms = 0)`.
  2. Áp dụng hệ số co giãn liên tục $\alpha = \text{clamp}(\text{EMA\_IKI} / 150.0, 0.15, 1.0)$.
  3. Kích hoạt Fast-Path $1.0\text{ms} \sim 1.5\text{ms}$ khi $N=1$ (đổi dấu thanh/nguyên âm như `a` $\to$ `á`).
  4. Đặt sàn an toàn $1.0\text{ms} + N \times 0.5\text{ms}$ cho thao tác xóa đa ký tự $N \ge 2$ khi gõ lướt Burst Typing ($\text{EMA\_IKI} \le 35\text{ms}$), tự động dãn về $10\text{ms} \sim 18\text{ms}$ khi gõ chậm.
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/ack-sensors/` (`ack-sensor.h`, `niri-sensor.h`, `generic-sensor.h`), `fcitx5-lilypad/src/lilypad-state.cpp`.

### [2026-08-25] Quyết định 017: Modular IIkiSensor Architecture & Passive Finger Speed Tracking
- **Bối cảnh:** Để tiến tới chuẩn Zero-Latency khi máy mượt và Zero-Corruption khi máy lag, bộ gõ cần biết tốc độ gõ ngón tay thực tế (Inter-Keystroke Interval - IKI) của người dùng để đóng vai trò ngòi nổ điều khiển (Gatekeeper), thay vì chỉ biết độ trễ đường truyền của Compositor/App (`IAckSensor`).
- **Quyết định:**
  1. Xây dựng Module cảm biến ngón tay độc lập `IIkiSensor`, `StandardIkiSensor` và `IkiSensorFactory` tại `fcitx5-lilypad/src/iki-sensors/`.
  2. Đo thụ động (Passive Listening 100%) $\Delta t = T_n - T_{n-1}$ giữa 2 lần nhấn phím vật lý liên tiếp trong `keyEvent()`, loại trừ phím Backspace giả lập uinput.
  3. Áp dụng thuật toán làm mịn EMA: $\text{IKI}_{\text{new}} = 0.35 \times \Delta t + 0.65 \times \text{prev\_EMA}$, lọc bỏ khoảng nghỉ dài $> 1000\text{ms}$ và micro-glitch $< 5\text{ms}$.
  4. Bổ sung cờ cấu hình an toàn `enableIkiAdaptive`, `ikiMinMs` (10ms), `ikiMaxMs` (500ms) trong `lilypad-config.h`.
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/iki-sensors/` (`iki-sensor.h`, `standard-iki-sensor.h`, `iki-sensor-factory.h`), `lilypad-state.h/.cpp`, `lilypad-config.h`.

### [2026-08-05] Quyết định 013: Modular IAckSensor Architecture & Universal Wayland Protocol
- **Bối cảnh:** Các Compositor (Niri, Sway, Hyprland, KWin, Mutter) phát tín hiệu Wayland Frame ACK khác nhau, việc viết gộp vào Sequencer Core làm mã nguồn bị phình to và không đo được độ trễ thực tế.
- **Quyết định:**
  1. Tách lớp cảm biến thành các Module C++ cắm/rút (`IAckSensor`, `NiriAckSensor`, `GenericAckSensor`, `AckSensorFactory`).
  2. Tự động kiểm tra `$XDG_CURRENT_DESKTOP` để nạp đúng Module cảm biến phù hợp.
  3. `GenericAckSensor` đóng vai trò cảm biến Vạn năng (Universal) xử lý 100% các Distro & Compositor hỗ trợ `zwp_input_method_v1/v2`.
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/ack-sensors/` (`ack-sensor.h`, `niri-sensor.h`, `generic-sensor.h`, `sensor-factory.h`).

### [2026-08-05] Quyết định 010: Dynamic Adaptive ACK Sensor & Safety Timeout (250ms Cap)
- **Bối cảnh:** Khi ứng dụng Web/Electron (Messenger, Facebook Post) bị lag/jank DOM, rào chắn ACK cần tự động mở rộng để tạo khoảng thở an toàn cho phím gõ tiếp theo.
- **Quyết định:**
  1. Bắt đầu bấm giờ $T_1$ (`start_time_ = steady_clock::now()`) khi phím được nạp vào Sequencer.
  2. Dừng bấm giờ $T_2$ và đo `elapsed` thực tế khi token xóa được nuốt sạch và commit string nổ.
  3. Kích hoạt `calculate_adaptive_delay_ms(elapsed)` nạp `elapsed` vào công thức `clamp` để tính toán Dynamic Barrier cho phím tiếp theo.
  4. Đặt rào chắn Safety Timeout trần **250ms** trong `lilypad-sequencer.h/.cpp`. Nếu app bị đóng băng quá 250ms, Sequencer tự động xả rào chắn tránh treo phím.
- **Mã nguồn thực thi:** [fcitx5-lilypad/src/lilypad-sequencer.cpp:L61-L75](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.cpp#L61-L75).

---

## ⚡ 2. BỘ NÃO ĐIỀU PHỐI SEQUENCER LAYER & QUẢN LÝ HÀNG ĐỢI

### [2026-08-09] Quyết định 016: Replay Micro-delay Fine Tuning (`isSpace ? 3000 : 300`) & Embedded Bamboo-Core
- **Bối cảnh:** Khi gõ siêu tốc phím chữ không bấm Space, khoảng hoãn $0.1\text{ms}$ cũ quá ngắn so với tốc độ render của App, gây lỗi đè rác chữ `cháau1kh`. Đồng thời, submodule pointer làm `git clone` thiếu file `go.mod`.
- **Quyết định:**
  1. Tích hợp trực tiếp 100% 19 file mã nguồn Go của `bamboo-core` vào repository (`fcitx5-lilypad/bamboo/bamboo-core/`), gỡ bỏ hoàn toàn Submodule pointer rỗng.
  2. Điều chỉnh `replay_delay_us` trong `lilypad-state.cpp` thành `isSpace ? 3000 : 300` (3ms cho Space, 0.3ms cho phím chữ).
  3. Cấu hình `LILYPAD_INFO` và `LILYPAD_DEBUG` thành `((void)0)` khi `NDEBUG` được bật (bản Release `-bin` công khai trên AUR/PPA tự động sạch 100% log, 0% overhead).
  4. Sửa `Library=liblilypad` và `[Dependencies]` trong `lilypad-addon.conf.in.in` và đặt `DESTINATION /lib/systemd/system` trong `misc/CMakeLists.txt`.
- **Mã nguồn thực thi:** `fcitx5-lilypad/bamboo/bamboo-core/`, `lilypad-state.cpp:L465`, `lilypad-utils.h:L31-L35`, `lilypad-addon.conf.in.in`, `misc/CMakeLists.txt`.

### [2026-08-05] Quyết định 014: Optimized Batch Replay Protocol (0.1ms Character vs 3ms Space Micro-gap)
- **Bối cảnh:** Khi tay gõ siêu tốc trong lúc rào chắn đang bật, các phím đệm `buffered_keys_` bị hoãn $15\text{ms}$ cho từng phím con trong vòng lặp đệ quy, gây dồn tích độ trễ lên tới 4.5 giây (`Typing so fast, add key to queue`).
- **Quyết định:**
  1. Phân biệt loại phím đệm khi xả hàng đợi: Phím ký tự thường (`a, b, c...`) xả tức thì $0.1\text{ms}$ (Batch Flush) bẻ gãy bẫy đệ quy.
  2. Riêng phím CÁCH (`Space`): Giữ nhịp ngắt vi mô $3\text{ms}$ để tách gói IPC an toàn với Chromium/Facebook.
- **Mã nguồn thực thi:** [fcitx5-lilypad/src/lilypad-state.cpp:L463-L473](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-state.cpp#L463-L473).

### [2026-08-05] Quyết định 011: Stale Serial Microstep Pruning (`serial < active_serial_`)
- **Bối cảnh:** Khi tay gõ nhanh liên tiếp, các vi bước (`MicroStep`) của phím cũ còn tồn đọng ở đầu hàng đợi `queue_` có thể nổ chen ngang vào phím mới gây ra lỗi lặp rác chữ (`mminimln`, `choa`).
- **Quyết định:**
  1. Mỗi giao dịch mới được gán một Serial ID nguyên tử tăng dần (`active_serial_`).
  2. Hàm `poll_next_step()` chạy vòng lặp `while (queue_.front().serial < active_serial_)` để phát hiện và vứt bỏ (`pop_front()`) toàn bộ các vi bước cũ hết hạn.
- **Mã nguồn thực thi:** [fcitx5-lilypad/src/lilypad-sequencer.cpp:L99-L104](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-sequencer.cpp#L99-L104).

---

## 🛠️ 3. HẠ TẦNG KERNEL UINPUT SERVER & XÓA VI MÔ

### [2026-08-05] Quyết định 012: Proportional Backspace Micro-delay & Micro-replacement (`deletedPart`)
- **Bối cảnh:** Việc xóa toàn bộ từ rồi gõ lại gây ra độ trễ lớn và làm giật con trỏ trên Web DOM.
- **Quyết định:**
  1. So sánh chuỗi cũ và mới qua `compareAndSplitStrings` để chỉ xóa phần hậu tố tối thiểu (`deletedPart`).
  2. Giới hạn trần số phím xóa `bsCount` bằng độ dài từ cũ (`maxBs = utf8::length(oldPreBuffer_)`) bảo vệ ranh giới từ.
  3. Áp dụng micro-delay tỷ lệ thuận với số phím xóa: `micro_delay_us = 6000 + bsCount * 4000` (1bs=10ms, 2bs=14ms, 3bs=18ms).
- **Mã nguồn thực thi:** [fcitx5-lilypad/src/lilypad-state.cpp:L519-L532](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-state.cpp#L519-L532).

### [2026-07-28] Quyết định 014 (Gốc 079): Pure Kernel Uinput Backspace Emission
- **Bối cảnh:** Việc sử dụng API `deleteSurroundingText()` gây xung đột dữ liệu và nhảy con trỏ trên các ứng dụng Electron và Web Editors.
- **Quyết định:**
  1. Nghiêm cấm 100% `deleteSurroundingText()` và `Preedit` trong Sequence Mode.
  2. 100% phím xóa được phát qua `/dev/uinput` server daemon dưới dạng mảng 4 sự kiện nguyên tử `ev[4]` (Press + SYN_REPORT + Release + SYN_REPORT) trong 1 lệnh `write()` duy nhất.
- **Mã nguồn thực thi:** [fcitx5-lilypad/server/lilypad-server.cpp:L72-L85](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/server/lilypad-server.cpp#L72-L85).

---

## 🚀 4. TÍCH HỢP FCITX5 FRAMEWORK & SYSTEMD USER SERVICE

### [2026-08-04] Quyết định 008: Chế Độ Gõ `Sequence` & Wayland IPC Keyboard Grab
- **Bối cảnh:** Cần thêm chế độ gõ chuyên biệt tích hợp Sequencer Layer trên giao diện cấu hình GUI và backend C++.
- **Quyết định:**
  1. Thêm `LilypadMode::Sequence` (Enum ID #9) vào `lilypad-config.h` và `lilypad-state.cpp`.
  2. Kích hoạt Wayland IPC Keyboard Grab (`enable_keyboard_grab = true`) đón phím mượt mà.
- **Mã nguồn thực thi:** `fcitx5-lilypad/src/lilypad-config.h`, `lilypad-state.cpp`.

### [2026-08-08] Quyết định 015: Lilypad Green Leaf Vector Brand Identity & Open-Source AI Kit Security
- **Bối cảnh:** Cần phân tách 100% nhận diện thương hiệu giữa Lotus (Hoa Sen cũ) và Lilypad (Lá Súng mới), đồng thời bảo mật bộ công cụ AI Kit (`.agent/`) khi đẩy mã nguồn công khai lên GitHub.
- **Quyết định:**
  1. Thay thế bộ icon SVG vector hoa sen cũ bằng bộ **Lá Súng xanh (Lilypad Leaf)** tươi mát trong `fcitx5-lilypad/data/icons/` (`fcitx-lilypad.svg`, `fcitx-lilypad-default.svg`, `fcitx-lilypad-off.svg`, `fcitx-lilypad-emoji.svg`).
  2. Đặt `useLilypadIcons = true` mặc định trong `lilypad-config.h` để Fcitx5 luôn hiển thị icon Lá Súng xanh trên Popup switcher và Status Bar.
  3. Cập nhật trang About (`about.py`) hiển thị chính xác phiên bản `2.2.0 (Stable)` cùng định dạng HTML chống xén chữ.
  4. Loại bỏ 100% mật khẩu cá nhân hardcoded trong `.agent/` và `.fcitx5-lilypad-ai/`, thay bằng cú pháp tổng quát `echo <SUDO_PASSWORD> | sudo -S <command>` để sẵn sàng cho Contributor dùng AI khi phát triển dự án công khai.
- **Mã nguồn thực thi:** `fcitx5-lilypad/data/icons/`, [fcitx5-lilypad/src/lilypad-config.h:L231](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/lilypad-config.h#L231), [settings-gui/ui/pages/about.py](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/settings-gui/ui/pages/about.py), `.agent/`.

### [2026-08-04] Quyết định 007: Thương Hiệu Độc Lập `fcitx5-lilypad` & Systemd Template Service
- **Bối cảnh:** Tách độc lập bộ gõ thành gói `fcitx5-lilypad` (`liblilypad.so`), bảo tồn `fcitx5-lotus-main` làm reference backup.
- **Quyết định:**
  1. Tạo Systemd Template Unit Service `fcitx5-lilypad-server@.service` tự động kích hoạt theo `$USER` hệ thống (`sudo systemctl enable --now fcitx5-lilypad-server@$USER.service`).
  2. Bổ sung parse UID (`-u username|uid`) trong `lilypad-server.cpp` và xác thực an toàn IPC `SO_PEERCRED` (`cred.uid == expected_uid`).
- **Mã nguồn thực thi:** [fcitx5-lilypad/server/lilypad-server.cpp:L179-L197](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/server/lilypad-server.cpp#L179-L197), [misc/fcitx5-lilypad-server@.service](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/misc/fcitx5-lilypad-server@.service).

---

## 🛡️ 5. CHÍNH SÁCH ỨNG DỤNG ĐẶC THÙ (AFFiNE / ELECTRON 39)

### [2026-08-05] Quyết định 044: AFFiNE Spurious Focus Analysis & Zero-Regression Rollback Policy
- **Bối cảnh:** AFFiNE (Canvas/Shadow DOM Editor) phát tín hiệu `InputContextFocusIn` / `activate` ngầm 10ms sau mỗi lần `commitString()`. Thử nghiệm can thiệp code C++ riêng cho AFFiNE đã làm ảnh hưởng (regression) đến các ứng dụng khác.
- **Quyết định:**
  1. Thực hiện **Zero-Regression Rollback**: Không viết bất kỳ dòng code hack C++ riêng nào cho AFFiNE bên trong bộ gõ, giữ mã nguồn C++ sạch 100%.
  2. Hướng dẫn người dùng cấu hình cờ Electron Wayland IME trong `~/.config/affine-flags.conf` (`--ozone-platform=wayland`, `--enable-wayland-ime`).
- **Mã nguồn thực thi:** Quy tắc bảo vệ toàn cục `Safety-First & Reversion Protocol`.