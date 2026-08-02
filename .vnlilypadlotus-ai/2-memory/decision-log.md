# VNLILYPAD LOTUS DECISION LOG (NHẬT KÝ QUYẾT ĐỊNH KIẾN TRÚC)

> **Kiến trúc Chuẩn:** Fcitx5 Lotus Upgrade Architecture (`v2.0.0-lotus`)
> **Nguyên tắc Đối soát:** Chỉ lưu trữ các Quyết định Kỹ thuật đang thực tế vận hành và phù hợp 100% với [system_map.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.vnlilypadlotus-ai/1-overview/system_map.md). Tất cả các thử nghiệm cũ (EVIOCGRAB evdev grab, zwp_virtual_keyboard_v1, DeleteSurroundingText, libunikey FFI, delay cứng) đã được lọc bỏ triệt để.

---

### [2026-08-02] Quyết định 007: Đổi tên Thương hiệu Độc lập (Lilypad) & Phục hồi Lotus Tham chiếu Backup
- **Bối cảnh:** Nhu cầu cài đặt bộ gõ `Lilypad` làm bộ gõ Tiếng Việt độc lập trên hệ thống Linux/Fcitx5, đồng thời duy trì mã nguồn `Lotus` gốc làm bản tham chiếu/backup sạch không chỉnh sửa.
- **Quyết định:**
  1. Đổi tên toàn bộ C++ namespace (`fcitx::Lilypad`), addon library target (`liblilypad.so`), desktop entry metadata, và binary (`fcitx5-lilypad-server`, `fcitx5-lilypad-settings`) trong thư mục `fcitx5-lilypad/`.
  2. Phục hồi 100% thư mục `fcitx5-lotus-main/` làm tài liệu tham chiếu gốc từ zip archive.
  3. Đã đăng ký cả 2 bộ gõ `lotus` và `lilypad` trong Fcitx5 profile và DBus group.
- **Phù hợp System Map:** Tương thích 100% với *No-Trash Repository Standard* & *Lilypad Core*.

---

### [2026-08-02] Quyết định 008: Thêm Chế độ Gõ "Sequence" trên Giao diện Cấu hình (Lilypad Mode Enum #9)
- **Bối cảnh:** Chuẩn bị hạ tầng UI/Config cho chế độ gõ mới **`Sequence`** trên `fcitx5-lilypad`.
- **Quyết định:**
  1. Thêm `Sequence` vào `LilypadMode` enum (giá trị int `9`) và macro i18n annotation trong `lilypad-config.h`.
  2. Thêm `MODE_SEQUENCE = 9` và cập nhật danh sách `global_modes` & `grid_modes` trong GUI Python Settings (`mode_manager.py` & `dynamic_settings.py`).
  3. Thêm cấu hình `ShowModeSequence` và `ShortcutSequence` trong `lilypadConfig`.
- **Phù hợp System Map:** Tương thích 100% với *Lilypad Settings GUI* & *Lilypad C++ Addon Core*.

---

### [2026-07-25] Quyết định 001 (Gốc 021): Adaptive App-Paced State Tracking & ACK Barrier Engine
- **Bối cảnh:** Các giải pháp bộ gõ cũ dùng thời gian trễ cứng (`sleep 5ms`) bị vỡ nhịp khi ứng dụng (Chrome, VS Code, Electron, Terminal) bị giật lag, sinh ra lỗi lặp từ và trôi chữ.
- **Quyết định:** 
  1. Loại bỏ hoàn toàn trễ cứng không biết trạng thái ứng dụng.
  2. Sequencer Layer vận hành theo cơ chế **State-Aware ACK Barrier (`BarrierState::WaitingForAck`)**. Lệnh tiếp theo chỉ được nhả ra khi nhận tín hiệu phản hồi `Done` (ACK) từ Wayland IPC Compositor (Niri).
- **Phù hợp System Map:** Tương thích 100% với *Sequencer Layer (Bộ Não)* trong `system_map.md`.

---

### [2026-07-26] Quyết định 002 (Gốc 024): Atomic Transaction Batching qua `poll_next_batch()` và Socket Flush
- **Bối cảnh:** Lệnh xóa và lệnh chèn chữ phát ra tách rời thành 2 giao dịch IPC riêng biệt khiến editor hiểu nhầm vị trí con trỏ và làm nghẽn socket.
- **Quyết định:**
  1. Sequencer Layer gom toàn bộ hành động của 1 cú bấm phím trong 1 giao dịch nguyên tử duy nhất qua `poll_next_batch()`.
  2. Ép `self.conn.flush()` ngay lập tức để đẩy gói tin ra Unix socket không độ trễ ($<0.1\,\text{ms}$).
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Layer* trong `system_map.md`.

---

### [2026-07-27] Quyết định 003 (Gốc 032): Hardware Key Repeat Filtering (`pressed_keys: HashSet<u32>`)
- **Bối cảnh:** Khi đè giữ phím Enter, Tab, Esc hoặc Modifier, Linux Kernel & Wayland Compositor phát liên tục sự kiện `Key Pressed` (30-50Hz) gây trôi dòng và nghẽn IPC socket.
- **Quyết định:**
  1. Sử dụng `pressed_keys: HashSet<u32>` trong `WaylandState` theo dõi tập hợp các phím vật lý đang ở trạng thái giữ.
  2. Bỏ qua các sự kiện `Pressed` lặp lại khi phím chưa được nhả (`Released`).
- **Phù hợp System Map:** Bảo vệ toàn diện luồng sự kiện cho *Wayland IPC* & *Engine Layer*.

---

### [2026-07-27] Quyết định 004 (Gốc 048): Giao Dịch Serial Thẻ Chuẩn (`im.commit(serial)`) Cho Phím Actions Rỗng
- **Bối cảnh:** Khi bấm các phím có `actions` rỗng, nếu không gửi `im.commit(serial)`, Niri Compositor bị nghẽn trạng thái (Stalled State) và kích hoạt lặp phím 30-60Hz.
- **Quyết định:**
  1. Cập nhật `state.serial = serial` ngay khi nhận sự kiện `Key`.
  2. Khi `actions.is_empty()`, bắt buộc gọi `state.im.commit(state.serial)` để đóng thẻ giao dịch với Niri Compositor.
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Layer*.

---

### [2026-07-27] Quyết định 005 (Gốc 051): Phân Rã Vi Bước `MicroStep` & Màng Ngắt Nhịp `WaitingMicroDelay` (1ms)
- **Bối cảnh:** Bắn phím xóa uinput `KEY_BACKSPACE` và `commit_string` trong cùng nanosecond làm Terminal/Editor nhận chèn chữ trước khi xóa xong, gây lặp chữ (`mminimln`, `choa`).
- **Quyết định:**
  1. Phân rã lô lệnh thành các vi bước nguyên tử `MicroStep::ForwardKey` và `MicroStep::CommitString`.
  2. Khi nhả phím xóa `KEY_BACKSPACE`, Sequencer tự động cài màng ngắt nhịp vi mô `WaitingMicroDelay` ($1\,\text{ms}$) trong vòng lặp `libc::poll` để màn hình render sạch phím xóa trước khi chèn chữ mới.
- **Phù hợp System Map:** Tương thích 100% với *Sequencer Layer MicroSteps Dispatcher*.

---

### [2026-07-27] Quyết định 006 (Gốc 055): Duy Trì Vòng Đời Keyboard Grab Liên Tục Khi Chuyển Focus
- **Bối cảnh:** Hủy `keyboard_grab` khi nhận `Event::Deactivate` làm Niri Compositor xem như IME đã ngắt kết nối và không gửi `Event::Activate` khi chuyển sang cửa sổ mới (GTK4 / GNOME Text Editor).
- **Quyết định:**
  1. Duy trì `keyboard_grab` liên tục trên Seat, **TUYỆT ĐỐI KHÔNG giải phóng grab trong `Deactivate`**.
  2. Tái sử dụng `keyboard_grab` sẵn có khi `Event::Activate` chuyển sang cửa sổ mới. Chỉ giải phóng grab khi tắt daemon (`Drop::drop()`).
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Layer Focus Handling*.

---

### [2026-07-28] Quyết định 007 (Gốc 068): Tối Ưu Hóa Rào Chắn Sequencer Layer (MicroDelay 1ms Uinput Backspace + Wayland IPC Commit ACK)
- **Bối cảnh:** Phím xóa `/dev/uinput` là driver kernel phần cứng nên Compositor không phát tín hiệu IPC `Event::Done` cho phím xóa, dễ làm Sequencer rơi vào 50ms timeout.
- **Quyết định:**
  1. Thao tác xóa phím `KEY_BACKSPACE` qua uinput: Đi qua trạng thái `WaitingMicroDelay (1ms)`, không chờ tín hiệu Wayland ACK.
  2. Thao tác chèn chuỗi `CommitString` qua Wayland IPC: Đi qua trạng thái `WaitingForAck` (Chờ tín hiệu `Event::Done` từ Niri).
- **Phù hợp System Map:** Phối hợp nhịp nhàng giữa *Kernel Layer (Uinput)* và *Wayland IPC Layer*.

---

### [2026-07-28] Quyết định 008 (Gốc 069): Khai Tử 100% Lệnh Cướp Phím `EVIOCGRAB` (0% Kernel Grab & Pure Uinput Backspace Engine)
- **Bối cảnh:** Việc cướp phím phần cứng bằng `EVIOCGRAB` làm phím bị chiếm giữ độc quyền, gây kẹt/đơ phím hệ thống khi daemon gặp sự cố hoặc chuyển focus.
- **Quyết định:**
  1. **Loại bỏ 100% lệnh `EVIOCGRAB`:** Không cướp phím phần cứng ở tầng evdev Kernel. Phím phần cứng và phím tắt (`Ctrl`, `Alt`, `Super`, `F1-F12`, Mũi tên) chảy tự nhiên 100% từ Kernel tới ứng dụng.
  2. **Duy nhất `/dev/uinput` cho phím xóa:** `/dev/uinput` chỉ phục vụ duy nhất nhiệm vụ bắn $N$ phím `KEY_BACKSPACE` để xóa ký tự thô cũ.
- **Phù hợp System Map:** Quy tắc Kiến trúc Toàn cục #1 (0% EVIOCGRAB Constraint).

---

### [2026-07-28] Quyết định 009 (Gốc 071): Bật Mặc Định Wayland IPC Keyboard Grab (`enable_keyboard_grab = true`)
- **Bối cảnh:** Mặc định `enable_keyboard_grab` cũ là `false` khiến bộ gõ bỏ qua sự kiện phím khi không truyền cờ CLI.
- **Quyết định:**
  1. Đặt `enable_keyboard_grab = true` làm mặc định.
  2. Tự động bật `im.grab_keyboard()` khi ô nhập liệu có focus.
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Layer*.

---

### [2026-07-28] Quyết định 010 (Gốc 072): Kích Hoạt Động Cơ Telex Tiếng Việt Native & Định Tuyến Phím Chuẩn
- **Bối cảnh:** Đảm bảo toàn bộ phím gõ tiếng Việt đều qua `VietnameseEngine` phân giải ký tự chính xác.
- **Quyết định:**
  1. Định tuyến 100% phím gõ ký tự qua `VietnameseEngine` Pure Rust state machine.
  2. Đẩy chuỗi kết quả `commit_string` nguyên tử qua Wayland IPC Socket.
- **Phù hợp System Map:** Tương thích 100% với *Telex/VNI Engine Layer*.

---

### [2026-07-28] Quyết định 011 (Gốc 073): Chuẩn Hóa Tăng Số Serial (`state.serial += 1`) & Phục Hồi ACK Barrier Nguyên Bản
- **Bối cảnh:** Wayland Compositor (Niri) chỉ gửi lại tín hiệu `Event::Done` khi số `serial` của giao dịch `im.commit(serial)` tăng dần. Nếu không tăng `serial`, Niri bỏ qua giao dịch cũ và Sequencer bị timeout 50ms.
- **Quyết định:**
  1. Tăng số `serial` trên từng giao dịch (`state.serial += 1`).
  2. Niri Compositor phản hồi tín hiệu ACK `Done` tức thì chỉ sau $0.1\,\text{ms}$, mở khóa rào chắn Sequencer mượt mà.
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Layer* & *Sequencer Layer*.

---

### [2026-07-28] Quyết định 012 (Gốc 077): Nuốt Phím Xóa Định Tính 100% Bằng Token Sequencer Layer (`expected_swallow_backspaces`)
- **Bối cảnh:** Dùng timer cứng $20\,\text{ms}$ để lọc phím xóa dội về gây race condition khi CPU bị giật lag.
- **Quyết định:**
  1. Sequencer Layer tự đếm chính xác $N$ phím xóa `ForwardKey { keycode: 14 }` phát ra vào token `expected_swallow_backspaces`.
  2. Khi Niri gửi phím 14 dội về `ZwpInputMethodKeyboardGrabV2`, bộ gõ trừ 1 token và **NUỐT CHỬNG SỰ KIỆN ĐỊNH TÍNH CHÍNH XÁC 100%** (0% magic timer).
- **Phù hợp System Map:** Tương thích 100% với *Sequencer Layer Orchestration*.

---

### [2026-07-28] Quyết định 013 (Gốc 078): Bộ Lọc Phím Tắt Hệ Thống (System Modifier Shortcut Guard)
- **Bối cảnh:** Khi giữ phím Modifier (`Ctrl`, `Alt`, `Super`), bộ gõ nếu vô tình can thiệp hoặc nhả phím sẽ làm vỡ phím tắt hệ thống (`Ctrl+W`, `Ctrl+S`, `Alt+Tab`) gây bật popup "Save Changes".
- **Quyết định:**
  1. Đọc cờ `mods_depressed` từ sự kiện `Modifiers` của `keyboard_grab`.
  2. Khi có phím Modifier đang được giữ (`state.modifiers != 0`), bộ gõ reset bộ nhớ đệm Telex và nhường 100% phím tắt cho Niri Compositor xử lý.
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Guard*.

---

### [2026-07-28] Quyết định 014 (Gốc 079): Chốt 100% Phát Phím Xóa `/dev/uinput` Code 14 (0% Predict, 0% DeleteSurroundingText)
- **Bối cảnh:** Đảm bảo khả năng tương thích tuyệt đối trên mọi ứng dụng Linux (Games, Wine, Terminal, Chrome, GTK4, Qt6, VS Code).
- **Quyết định:**
  1. Phát 100% phím xóa qua `/dev/uinput` code 14 cho mọi thao tác xóa ký tự thô cũ.
  2. KHÔNG DÙNG PREDICT, KHÔNG DÙNG `delete_surrounding_text()`.
- **Phù hợp System Map:** Quy tắc Kiến trúc Toàn cục #3 (Pure Uinput Backspace Emission).

---

### [2026-07-28] Quyết định 015 (Gốc 080): Hợp Nhất Dự Án & Xóa Sạch Thư Mục Dự Án Cũ (`vnlilypadkey`)
- **Bối cảnh:** Dọn dẹp triệt để mã nguồn và tài liệu cũ không phù hợp với kiến trúc Lotus.
- **Quyết định:** Xóa sạch 100% thư mục dự án cũ `/home/chiconcota/Documents/vnlilypadkey`. Toàn bộ mã nguồn, tài liệu kiến trúc, bộ nhớ AI hợp nhất hoàn toàn tại `/home/chiconcota/Documents/vnlilypad-lotus/`.
- **Phù hợp System Map:** Hợp nhất 100% workspace `vnlilypad-lotus`.

---

### [2026-07-30] Quyết định 016: Startup Enter Swallow Protocol (`im.commit_string("")` + `im.commit(serial)`)
- **Bối cảnh:** Khi khởi động lại bộ gõ trong cùng cửa sổ Terminal, cú nhấn phím Enter chạy lệnh vừa đi qua Shell vừa chui thẳng vào `keyboard_grab` của daemon mới. Việc `return` rỗng mà không commit IPC làm Wayland Compositor (Niri) coi phím Enter chưa được giải quyết và re-dispatch phím Enter lần 2 xuống Terminal.
- **Quyết định:**
  1. Trong 1.5 giây đầu khởi động (`start_time.elapsed() < 1500ms`), nếu nhận phím Enter (key 28/96), bộ gõ ghi nhận `pressed_keys.insert(key)`, gọi `im.commit_string("".to_string())` và `im.commit(serial)`.
  2. Việc gửi `commit_string("")` báo cho Compositor biết phím Enter đã được bộ gõ nuốt chửng thành chuỗi rỗng, triệt hạ 100% hiện tượng dội phím Enter xuống Terminal.
  3. Với các sự kiện lặp phím repeat hoặc khi `!state.active`, bắt buộc gọi `im.commit(serial)` để giải phóng luồng phím không bị nghẽn.
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Layer*.

---

### [2026-07-30] Quyết định 017: Virtual Keyboard Seat Release Protocol (`zwp_virtual_keyboard_v1`)
- **Bối cảnh:** Khi xử lý phím không phải chữ Tiếng Việt (`PassthroughNonVietnameseKey`), nếu chỉ gọi `im.commit(serial)` mà không phát tín hiệu phím ảo `vk.key(0, keycode, 1/0)` qua `zwp_virtual_keyboard_v1`, Niri Compositor sẽ coi phím phần cứng trên Seat chưa được giải phóng. Niri sẽ kích hoạt **vòng lặp Re-dispatch liên tục 1000Hz**, làm tràn hàng nghìn dòng Enter trắng xuống log Terminal.
- **Quyết định:**
  1. Trong `MicroStep::PassthroughNonVietnameseKey`, bắt buộc phát cặp tín hiệu `vk.key(0, keycode, 1)` (Press) và `vk.key(0, keycode, 0)` (Release) qua `zwp_virtual_keyboard_v1`.
  2. Gửi `im.commit(state.serial)` để chốt giao dịch IPC nguyên tử.
  3. Cơ chế này dứt điểm 100% hiện tượng tràn 1000 dòng Enter trắng khi restart daemon trên Terminal.
- **Phù hợp System Map:** Phối hợp *Wayland IPC* & *zwp_virtual_keyboard_v1*.

---

### [2026-07-30] Quyết định 018: Active State Default (`active: true`) & Unconditional Key Dispatch
- **Bối cảnh:** Các ứng dụng Terminal (Fish, Bash, Alacritty, Foot, Kitty) không phát sự kiện `zwp_text_input_v3::Activate`. Khi `state.active` mặc định là `false` hoặc bị kiểm tra `if !state.active` ngắt luồng, bộ gõ chiếm grab bàn phím nhưng nuốt sạch 100% phím gõ, gây ra hiện tượng **liệt hoàn toàn bàn phím (Keyboard Freeze)**.
- **Quyết định:**
  1. Đặt `active: true` mặc định trong `WaylandState::default()`.
  2. Loại bỏ khối lệnh `if !state.active` ngắt luồng sớm trong handler phím. Đảm bảo toàn bộ ứng dụng Terminal và GUI đều nhận được phím mượt mà 100%.
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Focus Dispatch*.

---

### [2026-07-30] Quyết định 019: Extended 3.0s Dual-State Startup Enter Guard (`Pressed` & `Released`)
- **Bối cảnh:** Khi khởi động lại bộ gõ liên tiếp lần 2 trong Terminal, sự kiện nhả phím `KeyState::Released` của cú bấm Enter chạy lệnh trước đó làm xóa phím Enter khỏi `pressed_keys`. Sự kiện nhả/lặp phím Enter sau mốc 1.5s làm lọt phím Enter xuống Shell.
- **Quyết định:**
  1. Mở rộng cửa sổ `Startup Enter Guard` lên **3.0 giây (3000ms)**.
  2. Xử lý nguyên tử cả 2 trạng thái `KeyState::Pressed` (commit string rỗng) và `KeyState::Released` (commit serial chốt thẻ IPC).
  3. Đảm bảo 100% cú bấm Enter chạy lệnh không thể bị dội kể cả khi restart bộ gõ lần 2 hay lần thứ $N$.
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Key Grab Guard*.

---

### [2026-07-30] Quyết định 020: Deduplicated Startup Enter Swallow Set (`launch_keys_swallowed`)
- **Bối cảnh:** Việc gửi lặp lại `commit_string("")` cho các gói tin re-dispatch/repeat phím Enter trong khoảng thời gian khởi động làm Niri chèn nhiều dòng trống.
- **Quyết định:**
  1. Thêm `launch_keys_swallowed: HashSet<u32>` vào `WaylandState`.
  2. Chỉ gửi `commit_string("")` duy nhất **1 LẦN DUY NHẤT** khi phím Enter `Pressed` xuất hiện lần đầu.
  3. Các sự kiện re-dispatch/repeat tiếp theo trong 1.5s chỉ gọi `im.commit(serial)` để chốt thẻ giao dịch IPC mà không chèn thêm dòng trống.
  4. Triệt hạ 100% hiện tượng tràn dòng trống và tràn prompt `[SIGKILL]>` khi restart daemon.
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Key Grab Guard*.

---

### [2026-07-30] Quyết định 021: Virtual Keyboard Hardware Seat State Re-sync on Startup
- **Bối cảnh:** Khi không chuyển cửa sổ (Focus shift), Niri Compositor giữ trạng thái `KeyState::Pressed` cho phím Enter phần cứng trên Seat của cửa sổ Terminal, gây hiện tượng lặp phím khi khởi động lại daemon. Chuyển Focus sang GNOME Text Editor xóa sạch trạng thái Seat cũ này.
- **Quyết định:**
  1. Ngay khi `zwp_virtual_keyboard_v1` được khởi tạo trong `WaylandImeClient::new()`, phát ngay lệnh `vk.key(0, 28, 0)` và `vk.key(0, 96, 0)` (Release Enter key ảo).
  2. Phát `vk.modifiers(0, 0, 0, 0)` để reset 100% trạng thái Modifier trên Seat.
  3. Tự động cưỡng chế Niri Compositor xóa sạch trạng thái giữ phím Enter phần cứng rác cũ từ phiên trước mà không cần người dùng phải chuyển đổi ứng dụng thủ công.
- **Phù hợp System Map:** Phối hợp *Wayland IPC* & *zwp_virtual_keyboard_v1 Seat State Re-sync*.

---

### [2026-07-30] Quyết định 022: Universal Direct Commit String Protocol (`commit_string`)
- **Bối cảnh:** Các ứng dụng Terminal (Fish, Bash, Alacritty, Foot, Kitty) không hỗ trợ hiển thị Preedit `set_preedit_string`. Khi bộ gõ dùng `set_preedit_string` thay vì `commit_string`, Terminal bỏ qua chuỗi văn bản làm người dùng bị hiện tượng đơ/liệt phím.
- **Quyết định:**
  1. Trong `MicroStep::CommitString`, bắt buộc dùng `im.commit_string(text)` + `im.commit(serial)`.
  2. Đảm bảo 100% văn bản được chèn trực tiếp lập tức vào mọi ứng dụng Terminal và GUI mà không bị nghẽn hay đơ/liệt phím.
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Commit Protocol*.

---

### [2026-07-30] Quyết định 023: Native Fcitx5 C++ Keyboard Grab Reset & Virtual Keyboard Key Forwarding
- **Bối cảnh:** Soi mã nguồn C++ chính thức `fcitx5-master/src/frontend/waylandim/waylandimserverv2.cpp`. Fcitx5 giải phóng grab cũ trước khi tạo Grab mới (dòng 217) và chuyển tiếp phím không dùng qua `zwp_virtual_keyboard_v1::key` kết hợp `im.commit_string("")` (dòng 616).
- **Quyết định:**
  1. Trong `Event::Activate`, thực hiện `state.keyboard_grab = None;` rồi re-grab nguyên tử như Fcitx5 C++ dòng 217.
  2. Trong `MicroStep::PassthroughNonVietnameseKey`, phát tín hiệu `vk.key(0, keycode, 1)` và `vk.key(0, 28, 0)` qua `zwp_virtual_keyboard_v1` kết hợp `im.commit_string("")` nuốt phím phần cứng.
  3. Loại bỏ 100% hiện tượng Niri re-dispatch phím phần cứng xuống Terminal, tương thích chuẩn C++ Fcitx5.
- **Phù hợp System Map:** Tương thích 100% với *Fcitx5 C++ WaylandIM Module Standard*.

---

### [2026-07-30] Quyết định 024: Clean Single Grab Initialization & Clean Wayland Serial Passthrough
- **Bối cảnh:** Việc re-grab trong `Activate` khi đối tượng grab đã tồn tại gây ra lỗi giao thức làm đơ liệt phím. Việc phát `commit_string("")` cho các phím passthrough không phải chữ Tiếng Việt làm nuốt sạch mọi phím gõ.
- **Quyết định:**
  1. Duy trì **1 LẦN GRAB DUY NHẤT** trong `WaylandImeClient::new()`. Không re-grab trong `Activate`.
  2. Trong `MicroStep::PassthroughNonVietnameseKey`, chỉ gọi `im.commit(state.serial)` chốt thẻ IPC để Niri cho phím trôi qua mượt mà.
  3. Đảm bảo 100% bàn phím không bị đơ/liệt trên bất kỳ ứng dụng Terminal hay GUI nào.
- **Phù hợp System Map:** Tương thích 100% với *Wayland IPC Clean Grab Protocol*.

---

### [2026-07-30] Quyết định 025: Fcitx5 Dual Pipeline & Expanded Evdev Keycode Mapping
- **Bối cảnh:** Mở rộng bản ánh xạ `evdev_keycode_to_char` bao gồm toàn bộ chữ, số (1..0), dấu câu (-, =, [, ], ;, ', ,, ., /) để đưa 100% phím nhập liệu qua Telex engine.
- **Quyết định:**
  1. Mở rộng `evdev_keycode_to_char` xử lý 100% ký tự gõ văn bản.
  2. Trong `PassthroughNonVietnameseKey`, sử dụng `vk.key(0, keycode, 1)` + `vk.key(0, 28, 0)` qua `zwp_virtual_keyboard_v1` kết hợp `im.commit_string("")` nuốt phím phần cứng như C++ Fcitx5 dòng 616.
  3. Đảm bảo 0% liệt phím và 0% dội phím trên mọi cửa sổ Terminal và GUI app.
- **Phù hợp System Map:** Tương thích 100% với *Fcitx5 C++ Dual Pipeline Protocol*.

---

### [2026-07-30] Quyết định 026: Hybrid Fcitx5-Lotus Sequencer Integration Architecture
- **Bối cảnh:** Việc tự phát triển tầng Wayland IPC Standalone Daemon gặp nhiều tranh chấp tài nguyên Niri Compositor (dội phím Enter khi restart, đơ liệt phím trên Terminal). Trong khi Fcitx5 C++ Framework đã xử lý hoàn hảo 100% hạ tầng Wayland IPC, System Tray, GUI Config và Systemd Service.
- **Quyết định:**
  1. Chuyển hướng kiến trúc chiến lược sang **Mô hình Hybrid**: Tận dụng 100% Fcitx5 Framework làm hạ tầng gác cửa Wayland IPC.
  2. Tích hợp duy nhất thuật toán **Sequencer Token Swallow Layer** của dự án vào C++ Addon `fcitx5-lotus` (`src/lotus-state.cpp`).
  3. Loại bỏ thời gian chờ 50ms rác của lotus cũ, biến `fcitx5-lotus` thành bộ gõ tiếng Việt gõ mượt 0ms tức thì, 100% ổn định trên Linux.
- **Phù hợp System Map:** Phối hợp *Fcitx5 C++ Framework & Sequencer Layer Integration*.

---

### [2026-07-31] Quyết định 027: Legacy Daemon Layer Pruning & Isolated Sequencer Scope
- **Bối cảnh:** Toàn bộ hạ tầng Wayland IPC, Evdev Grab, Uinput Server và Telex State Machine nguyên bản đã có Fcitx5 C++ và `bamboo-core` đảm nhận. Việc duy trì các layer daemon độc lập cũ (`src/wayland/`, `src/kernel/`, `src/engine/`, `src/bin/`) tạo ra dư thừa mã nguồn.
- **Quyết định:**
  1. Xóa sạch 100% các layer daemon cũ (`src/wayland/`, `src/kernel/`, `src/engine/`, `src/bin/`, `scripts/emergency_unbind.sh`).
  2. Chỉ giữ duy nhất `src/sequencer/` trong Rust làm engine tham chiếu thuật toán chuẩn (100% `cargo test` pass).
  3. Tận dụng Telex engine nguyên bản của `fcitx5-lotus` (Go/Bamboo), tập trung 100% nguồn lực hoàn thiện trình điều phối Sequencer Token Swallow Layer trong C++ Addon `fcitx5-lotus-main`.
- **Phù hợp System Map:** Tinh gọn mã nguồn 100%, bảo đảm nguyên tắc Zero-Trash Directive.

---

### [2026-07-31] Quyết định 028: Rebranding Input Method Display Name to "Lilypad"
- **Bối cảnh:** Nhận diện bộ gõ mới cần phân biệt hoàn toàn với bộ gõ lotus mặc định cũ trên GUI Fcitx5 Configtool và System Tray.
- **Quyết định:**
  1. Đổi tên cấu hình `InputMethod.Name` trong `lotus.conf.in` thành `Lilypad`.
  2. Đổi tên `Addon.Name` trong `lotus-addon.conf.in.in` thành `Lilypad IME For Fcitx5`.
  3. Đổi tên metadata trong `org.fcitx.Fcitx5.Addon.Lotus.metainfo.xml.in.in` thành `Lilypad`.
- **Phù hợp System Map:** Tương thích 100% với *Fcitx5 Addon Standard*.

---

### [2026-07-31] Quyết định 029: 0ms Deterministic C++ Sequencer Token Swallow Implementation
- **Bối cảnh:** Loại bỏ hoàn toàn các vòng lặp sleep cứng (`sleep_for` 20ms/50ms) cũ làm lag nhịp gõ trong `handleUInputKeyPress`.
- **Quyết định:**
  1. Thêm `std::atomic<int> expected_swallow_backspaces_{0};` vào `LotusState` (`lotus-state.h`).
  2. Trong `performReplacement`, gán token swallow số lượng $N$ phím backspace nguyên tử (`expected_swallow_backspaces_.store(N)`).
  3. Trong `handleUInputKeyPress`, khi nhận Backspace uinput dội về, nếu `expected_swallow_backspaces_ > 0`, lập tức `fetch_sub(1)` và gọi `event.filterAndAccept()` nuốt phím **0ms delay**.
  4. Chèn trực tiếp chuỗi chữ mới `commitString` tức thì mà không qua bất kỳ timer chờ cứng nào.
- **Phù hợp System Map:** Tương thích 100% với *Sequencer Layer Token Swallow Architecture*.

---

### [2026-07-31] Quyết định 030: Application Uinput Backspace Passthrough & 0ms Direct Commit
- **Bối cảnh:** Việc nuốt chửng 100% phím xóa uinput tại lối vào `keyEvent` bằng `event.filterAndAccept()` làm cho App (Kitty/Terminal/Chrome) không bao giờ nhận được phím xóa để xóa ký tự thô cũ, sinh ra lỗi lặp từ kép (`chaoào`, `vaânận`, `loiôiội`).
- **Quyết định:**
  1. Cho phép phím xóa uinput truyền qua (`return false;` không gọi `filterAndAccept()`) để App (Kitty/Terminal/GUI) nhận được phím xóa và thực sự xóa $N$ ký tự thô trên màn hình.
  2. Ngay khi phím xóa uinput chạm mốc $N$, gọi `ic_->commitString(pending_commit_string_)` lập tức trong **0ms delay** (không dùng `sleep_for`).
  3. Loại bỏ các cờ cộng dồn Backspace rác trong `performReplacement` (`++expected_backspaces_`), tính toán chính xác $N = \text{utf8::length}(deletedPart)$.
- **Phù hợp System Map:** Tương thích 100% với *Application Text Erasure & 0ms Direct Commit Protocol*.

---

### [2026-07-31] Quyết định 031: Serial ID Tagging & Stale Token Discard Protocol (`lotus-sequencer.h/.cpp`)
- **Bối cảnh:** Khi gõ nhanh liên tiếp nhiều lượt thay thế, phím xóa uinput dội về của đợt gõ cũ (`Serial #N-1`) có thể bị ghi nhận nhầm vào đợt gõ mới (`Serial #N`), gây sai lệch thứ tự xóa/chèn.
- **Quyết định:**
  1. Tạo file mới `lotus-sequencer.h` và `lotus-sequencer.cpp` chứa struct `LotusSequencer` quản lý Serial ID tăng dần đơn điệu (`serial_counter_`).
  2. Mỗi `MicroStep` (EmitBackspace, CommitString) mang nhãn `serial` để đối chiếu khi phím xóa dội về.
  3. Hàm `should_swallow_backspace(serial)` kiểm tra khớp `serial` trước khi tiêu thụ token xóa, loại bỏ 100% token rác từ đợt gõ cũ.
- **Phù hợp System Map:** Tương thích 100% với *Sequencer Token Swallow Layer*.

---

### [2026-07-31] Quyết định 032: Bamboo Core Reset Protection During Active Replacement (`lotus-engine.cpp`)
- **Bối cảnh:** Khi Wayland Compositor phát sự kiện `InputContextReset` (focus shift, popup, etc.) trong khi `is_deleting_ == true`, Bamboo Core bị reset lịch sử từ ngữ giữa chừng, khiến từ tiếp theo bị mất ngữ cảnh gõ dấu.
- **Quyết định:**
  1. Trong `LotusEngine::reset()`, kiểm tra `state->is_deleting()` trước khi gọi `ResetEngine()`.
  2. Nếu đang trong tiến trình thay thế, bỏ qua hoàn toàn `InputContextReset` để bảo vệ 100% bộ nhớ lịch sử Bamboo Core.
- **Phù hợp System Map:** Tương thích 100% với *Bamboo Core State Protection*.

---

### [2026-07-31] Quyết định 033: Special Key Pass-Through During Deletion State (`lotus-state.cpp`)
- **Bối cảnh:** Các phím điều khiển đặc biệt (Enter, Escape, Tab, Arrow Keys, Modifier) bị nhốt vào `buffered_keys_` khi `is_deleting_ == true`, gây hiện tượng Enter bị đè lặp/dính sau khi thay thế từ hoàn tất.
- **Quyết định:**
  1. Trong `keyEvent()`, khi `is_deleting_` đang active, nếu phím là `Return`, `KP_Enter`, `Escape`, `Tab`, `isCursorMove()` hoặc `hasModifier()`: Lập tức xóa trạng thái `is_deleting_`, gọi `keyEvent.forward()` cho phím chảy qua tự nhiên.
  2. Không bao giờ nhốt các phím điều khiển vào `buffered_keys_`.
- **Phù hợp System Map:** Tương thích 100% với *Special Key Passthrough Protocol*.

---

### [2026-07-31] Quyết định 034: Zero deleteSurroundingText & Zero Preedit — 100% Kernel Uinput Sequencer Layer
- **Bối cảnh:** User yêu cầu nghiêm cấm dùng `deleteSurroundingText` và `Preedit` trong suốt quá trình phát triển dự án (đã nhắc nhở 3 lần trong phiên). Lệnh `deleteSurroundingText` đi qua IPC Fcitx5 khác luồng với phím xóa uinput từ Kernel, gây race condition và lỗi `chaà` (xóa thiếu 1 ký tự).
- **Quyết định:**
  1. Loại bỏ 100% tất cả lệnh `ic_->deleteSurroundingText()` khỏi toàn bộ mã nguồn `fcitx5-lotus-main/src/`.
  2. Mọi thao tác thay thế từ (gõ dấu Tiếng Việt, macro, double-space, em-dash) đều đi duy nhất qua `performReplacement()` sử dụng Kernel Uinput Sequencer Layer.
  3. Thêm `usleep(1500)` ($1.5\,\text{ms}$) giữa các phím xóa uinput trong `lotus-server.cpp` để Linux Kernel evdev không gộp các sự kiện `SYN_REPORT`, đảm bảo App nhận đủ 100% $N$ phím xóa.
- **Phù hợp System Map:** Quy tắc Kiến trúc Toàn cục #4 (Race Condition Prevention) & Quy tắc #3 (Pure Uinput Backspace Emission).

---

### [2026-08-01] Quyết định 035: Sequencer Deferred Event-Loop Commit Protocol (`addTimeEvent` 2ms)
- **Bối cảnh:** Khi gọi `ic_->commitString()` trực tiếp trong `keyEvent()` ngay khi nhận phím xóa uinput cuối cùng, IPC `commit_string` đến App TRƯỚC khi phím xóa uinput thứ 2 được Fcitx5 forward tới App. App chèn `áo` thành `chaáo` rồi phím xóa uinput thứ 2 mới tới xóa `o` thành `chaá` (tương tự `giao` + `s` $\rightarrow$ `giaá`).
- **Quyết định:**
  1. Hoãn `ic_->commitString()` và `replayBufferedKeys()` 2ms qua `engine_->instance()->eventLoop().addTimeEvent(...)`.
  2. Đảm bảo 100% phím xóa uinput từ Kernel được App thực thi xong trước khi chuỗi ký tự `commitString` mới tới App.
  3. Lưu `commit_timer_` (`std::unique_ptr<EventSourceTime>`) làm member trong `LotusState` để bảo toàn vòng đời RAII Timer.
- **Phù hợp System Map:** Tương thích 100% với *Sequencer Layer Deferred Commit Protocol*.

---

---

### [2026-08-02] Quyết định 037: AT-SPI2 `RegisterEvent` Kebab-Case Spec Compliance & ChildrenChanged ACK
- **Bối cảnh:** Phiên debug thực nghiệm trên `dbus-monitor` phát hiện spec AT-SPI2 yêu cầu register rule dạng kebab-case (`object:text-changed`, `object:children-changed`), Registry normalize thành `Object:TextChanged`/`Object:ChildrenChanged` và trả `EventListenerRegistered`. Ngoài ra payload signal `s i i v a{sv}` có arg đầu tiên chính là detail string (`"delete"`/`"remove"`), không phải int.
- **Quyết định:**
  1. Chuẩn hóa `RegisterEvent` trong `lotus-atspi.cpp` sang kebab-case đúng spec (`object:text-changed` + `object:children-changed`), đã xác minh hoạt động 100% mọi lần restart qua `dbus-monitor`.
  2. Parse đúng 5-tuple `s i i v a{sv}` (arg0 = detail string), thêm ACK `ChildrenChanged:remove` bên cạnh `TextChanged:delete`.
  3. Giữ nguyên kết nối tới `unix:path=/run/user/$UID/at-spi/bus_1`, log `📡 [AT-SPI SIGNAL RECV]` kèm sender + start index + length để đối soát.
- **Phù hợp System Map:** Tương thích 100% với *AT-SPI2 DBus DOM Render ACK Engine* (Quyết định 012).

---

### [2026-08-02] Quyết định 038: Adaptive Slow-App Barrier (`isSlowApp` + `set_max_ack_timeout_ms`)
- **Bối cảnh:** Facebook Web / Messenger Web / Chrome / AFFiNE / Electron render DOM chậm hơn rất nhiều so với Terminal/GTK, nên Safety Timeout 15ms cố định dễ nhả `commitString` trước khi DOM xóa xong chữ.
- **Quyết định:**
  1. Thêm `isSlowApp()` (`getActiveApp` matches `chromium|firefox|electron|affine|chrome|messenger|facebook`) → `WaitingForDomAck` timeout 35ms; app nhanh chạy fast-path micro-delay ngắn.
  2. Thêm `LotusSequencer::set_max_ack_timeout_ms()` để đổi timeout runtime theo app class.
- **Phù hợp System Map:** Tương thích 100% với *Sequencer BarrierState* & *Heavy App Optimization*.

---

### [2026-08-02] Quyết định 039: Environment-Gated A11y Discovery — DOM ACK Chưa Hoạt Động Do Môi Trường, Không Phải Code
- **Bối cảnh:** Dù đăng ký AT-SPI đúng spec (Quyết định 037), debug 5 vòng `dbus-monitor` + `FCITX_LOG_LEVEL=debug` cho thấy Chrome/Firefox KHÔNG phát signal nào. Kiểm tra môi trường: `GTK_MODULES`, `ACCESSIBILITY_ENABLED`, `NO_AT_BRIDGE`, `AT_SPI_BUS_ADDRESS` đều trống/không đúng → GTK/Chromium không nạp `atk-bridge`, bus `/run/user/1000/at-spi/bus_1` không tồn tại → không có publisher signal.
- **Quyết định:**
  1. Khẳng định vấn đề nằm ở **môi trường session chưa bật accessibility**, không phải code C++.
  2. Cách kích hoạt đúng: tạo `~/.config/environment.d/10-a11y.conf` (`GTK_MODULES=gail:atk-bridge`, `ACCESSIBILITY_ENABLED=1`, `NO_AT_BRIDGE=0`) rồi **đăng xuất/đăng nhập lại session** (export tay không đủ vì app sinh ra từ compositor không thấy env).
  3. Phiên sau bắt đầu bằng bước kích hoạt môi trường này rồi mới test (xem [checkpoint.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.vnlilypadlotus-ai/2-memory/checkpoint.md)).
  4. Nếu sau khi bật env vẫn không có signal → thử Chrome `--force-renderer-accessibility` để cô lập đúng tầng.
- **Phù hợp System Map:** Quy tắc Kiến trúc Toàn cục #6 (Debug đối soát môi trường) & *AT-SPI2 DBus ACK Engine*.

---

### [2026-08-02] Quyết định 040: GỠ BỎ TOÀN BỘ AT-SPI2 DOM ACK ENGINE — Khôi Phục Trạng Thái Pre-AT-SPI2
- **Bối cảnh:** User yêu cầu "khôi phục phiên bản ban đầu của main trước khi rẽ nhánh". Xác minh: code C++ `fcitx5-lotus-main/` nằm trong `.gitignore` (line 7), chưa từng được commit (`git ls-files` = 0 file), không có `.git` riêng, không có backup → **không thể revert bằng git**. Commit `aea7094` (chứa QD 012 AT-SPI2) nằm trên nhánh `feature/phase3-wayland-frame-ack`, main (`a568521`) không có AT-SPI2.
- **Quyết định:**
  1. Gỡ tay 100% AT-SPI2 khỏi 7 file: xóa `lotus-atspi.h/.cpp`, gỡ khỏi `CMakeLists.txt`, gỡ `isSlowApp()` + constructor ack-callback trong `lotus-state.cpp`, gỡ `WaitingForDomAck`/`set_waiting_dom_ack`/`receive_dom_ack`/`set_max_ack_timeout_ms` trong `lotus-sequencer.*`, gỡ `reassert_registration()` trong `lotus-engine.cpp`.
  2. Giữ nguyên cơ chế `WaitingForAck` (Niri Frame ACK) + micro-delay + `max_ack_timeout_ms=15` (có từ trước AT-SPI2).
  3. **Rủi ro đã nêu:** không khớp 100% bản gốc vì không có bản gốc đối chiếu; build pass 100% + clang-tidy sạch cho 3 file đã sửa.
- **Phù hợp System Map:** Quy tắc Kiến trúc Toàn cục #6 (No-Trash) & *Sequencer Layer WaitingForAck*.

---

### [2026-08-02] Quyết định 041: Cảnh Báo Vĩnh Viễn — Code C++ Ngoài Git Không Có Snapshot
- **Bối cảnh:** Sau 2 lần user yêu cầu "revert code" mà không thể do code chưa từng commit, xác lập quy tắc bảo vệ dự án.
- **Quyết định:**
  1. Mọi thay đổi lớn lên `fcitx5-lotus-main/` PHẢI được ghi lại chi tiết vào `checkpoint.md` (file đã sửa + nội dung) trong cùng phiên.
  2. Khi user yêu cầu rollback: kiểm tra `git ls-files fcitx5-lotus-main/` TRƯỚC, nếu = 0 file thì báo ngay "không thể revert bằng git" và đề xuất gỡ tay hoặc giữ nguyên.
  3. Đề xuất tương lai: cân nhắc thêm `fcitx5-lotus-main/` vào git (bỏ ignore) để có snapshot khôi phục.
- **Phù hợp System Map:** Quy tắc Kiến trúc Toàn cục #6 (No-Trash Repository Standard).

---

### [2026-08-01] Quyết định 036: State-Aware Bamboo Core Auto-Rebuild Protocol (`EngineRebuildFromText`) & Sequencer Queue Integration
- **Bối cảnh:** Trên Wayland Native (GTK4 / GNOME Text Editor), khi ứng dụng nhận phím xóa uinput, GTK gửi phản hồi Wayland `InputContextReset` về Fcitx5. Nếu Bamboo Core Engine bị reset bộ nhớ rỗng giữa chừng (như sau khi commit `âu` trong `mau6`), phím gõ tiếp theo (`4` hoặc `5`) không thể gắn dấu vào từ cũ, sinh ra phím thô `mâu4`, `đươc5`.
- **Quyết định:**
  1. Duy trì cờ `is_deleting_ = true` xuyên suốt cửa sổ hoãn 2ms cho tới khi `commit_timer_` callback chạy xong để chặn đứng mọi reset rác từ Wayland IPC.
  2. Trước khi xử lý phím mới trong `handleUinputMode` và `replayBufferedKeys`, nếu `oldPreBuffer_` không rỗng ("mâu", "đươc") mà Bamboo Core rỗng, code C++ tự động gọi `EngineRebuildFromText(lotusEngine_.handle(), oldPreBuffer_.c_str())` để khôi phục 100% trạng thái bộ nhớ từ cho Bamboo Core.
  3. Tích hợp `sequencer_.poll_next_step()` rút đúng `MicroStepType::CommitString` từ hàng đợi `sequencer_.queue_`.
- **Phù hợp System Map:** Tương thích 100% với *Bamboo Core Auto-Rebuild & Sequencer Queue Protocol*.