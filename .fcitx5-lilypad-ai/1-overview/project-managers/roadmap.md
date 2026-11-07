# VNLILYPAD-LOTUS ROADMAP & IMPLEMENTATION PLAN

## 🚀 Kế hoạch Tích hợp Hybrid: `fcitx5-lotus` + Sequencer Layer (Serial ID Token Swallow)

### 🎯 Mục tiêu:
Biến `fcitx5-lotus` thành bộ gõ Tiếng Việt Telex mượt nhất, nhanh nhất trên Linux mà không cần phát minh lại bánh xe Wayland IPC.

---

### ✅ Đã Hoàn Thành (Phase 1 & Phase 2 — Sequencer V2 & Core Fixes):

- [x] **Bước 1:** Nghiên cứu mã nguồn C++ `fcitx5-lotus` (`lotus-state.cpp`, `lotus-state.h`).
- [x] **Bước 2:** Nhúng thuật toán Sequencer Token Swallow vào C++ (`lotus-sequencer.h/.cpp`).
- [x] **Bước 3:** Serial ID Tagging (`Serial #1, #2, ... #N`) chống lệch thứ tự token xóa.
- [x] **Bước 4:** Xóa sạch 100% `deleteSurroundingText` — ép 100% qua Kernel Uinput Sequencer.
- [x] **Bước 5:** Bamboo Core Reset Protection khi `is_deleting_`.
- [x] **Bước 6:** Special Key Pass-Through (Enter/Esc/Tab) khi `is_deleting_`.
- [x] **Bước 7:** Inter-Backspace Delay `usleep(1500)` trong `lotus-server.cpp`.
- [x] **Bước 8:** Sequencer Deferred Event-Loop Commit (`addTimeEvent` 2ms) — Triệt hạ lỗi `chaá`/`giaá`.
- [x] **Bước 9:** State-Aware Bamboo Core Auto-Rebuild (`EngineRebuildFromText`) — Triệt hạ lỗi `mâu4`/`đươc5` trên GTK Wayland.
- [x] **Bước 10:** Tích hợp Sequencer Queue Polling (`poll_next_step`) trực tiếp trong `handleUInputKeyPress`.

---

### 🟡 Kế Hoạch Đã Hoàn Thành (Phase 3 — Rebrand, Dual Mode & Packaging AUR):
- [x] Đổi tên thương hiệu độc lập **Lotus -> Lilypad** (`liblilypad.so`, `fcitx5-lilypad`, `fcitx5-lilypad-server`).
- [x] Phục hồi 100% mã nguồn `fcitx5-lotus-main/` từ zip archive làm bản tham chiếu gốc, giữ nguyên không sửa.
- [x] Đăng ký song song cả **Lotus** và **Lilypad** trên hệ thống và Fcitx5 active profile/DBus.
- [x] Thêm chế độ gõ **`Sequence`** (UI dropdown, C++ enum `#9`, Python Settings GUI) cho `fcitx5-lilypad`.
- [x] Tích hợp hạ tầng Modular ACK Sensor (`IAckSensor`, `NiriAckSensor`, `GenericAckSensor`, `SensorFactory`).
- [x] Tích hợp trực tiếp mã nguồn Go `bamboo-core` vào main repo, tối ưu nhịp replay `3000:300` và Zero-Log Overhead.
- [x] Xây dựng hạ tầng đóng gói AUR cho 3 phiên bản (`fcitx5-lilypad`, `fcitx5-lilypad-bin`, `fcitx5-lilypad-git`).

---

### 🚀 Kế Hoạch Đang Triển Khai (Phase 4 — IKI Adaptive Engine & Two-Tier Timeout Architecture):
> **Nhánh Git:** `feat/iki-adaptive-engine` | **Chi tiết:** Xem [iki-adaptive-engine-plan.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/1-overview/project-managers/iki-adaptive-engine-plan.md)

- [x] **Phase 4.1: Passive IKI Measurement (Đo IKI ngầm 0% Overhead)**
  - [x] Thêm biến đo $\Delta t = T_n - T_{n-1}$ trong `LilypadState::keyEvent()` qua `IIkiSensor`.
  - [x] Tính `current_iki_ms_` qua thuật toán EMA smoothing.
  - [x] Thêm cờ cấu hình `enableIkiAdaptive` trong `lilypad-config.h`.
- [x] **Phase 4.2: Dynamic Micro-Pacing Optimization**
  - [x] Điều chỉnh động `micro_delay_us` theo $\alpha = \text{clamp}(\text{EMA\_IKI} / 150.0, 0.15, 1.0)$.
  - [x] Ép nhịp micro-pacing về $1\text{ms} \sim 2.5\text{ms}$ khi gõ lướt siêu tốc (Zero-Latency feel).
- [x] **Phase 4.3: Two-Tier Timeout & Context Invalidation (State Protection)**
  - [x] Thiết lập **Soft Timeout** ($T_{\text{soft}} = 35\text{ms} \sim 120\text{ms}$): Giữ phím trong RAM (`buffered_keys_`), chuyển `BarrierState::AppLagHolding`.
  - [x] Thiết lập **Hard Timeout** ($250\text{ms}$): Watchdog timer trên EventLoop, cắt lỗ khẩn cấp `purgeContextEmergency()`, purge word buffer và xả phím thô an toàn khi app bị freeze.
- [x] **Kiểm thử thực tế & Hoàn thành Phase 4.3** (User verified: Thành công tốt đẹp trên cả Terminal lẫn Facebook/Chrome).
- [x] **Phase 5: Merge nhánh `feat/iki-adaptive-engine` vào `main`, nâng Semantic Version `v2.3.0` & Phát hành chính thức lên GitHub Releases + AUR**.
- [x] **Phase 5.1: Chuẩn hóa Hướng dẫn Cài đặt AUR 2 Bước, Cải tiến Scriptlet `.install` & Phát hành Bản vá v2.3.1**.
- [x] **Phase 5.2: Tri-Layer Dynamic Micro-Pacing, Loại bỏ Nợ Kỹ thuật & Chuẩn hóa Phát hành v2.3.6**:
  - [x] Tích hợp đo đạc thời gian nuốt phím thực tế ($\Delta T_{\text{swallow}}$), bảo vệ 3 tầng (Tri-Layer Protection).
  - [x] Loại bỏ hoàn toàn cờ `is_antigravity_flag_` và logic mức sàn cứng riêng, chuyển về cơ chế chuẩn Chromium/Electron.
  - [x] Chuẩn hóa ngưỡng Watchdog $250\text{ms}$ thống nhất toàn hệ thống.
  - [x] Đóng gói nhị phân Release `dist/fcitx5-lilypad-v2.3.6-x86_64-archlinux.tar.zst`, cập nhật SHA256 checksum và phát hành tag Git `v2.3.6`.
  - [x] Cập nhật đồng bộ 3 gói AUR: `fcitx5-lilypad`, `fcitx5-lilypad-bin`, `fcitx5-lilypad-git`.
  - [x] Sửa triệt để tác giả commit sang `chiconcota <lytatthanh@gmail.com>` và cập nhật đầy đủ lời cảm ơn trong 3 file README.

---

### 🦀 Đã Hoàn Thành (Phase 6 — Rust Core Engine Migration `bamboo-vi/bamboo-core`):
> **Mục tiêu:** Thay thế engine Go C-FFI bằng Rust thuần hiệu năng cao, giảm RAM về $<1\text{MB}$, nén binary, đạt Zero-Heap Allocation và $O(1)$ Backspace Snapshot.

- [x] **Phase 6.1: Khởi tạo nhánh `feat/rust-bamboo-engine` & Thiết lập Build Toolchain:**
  - [x] Nhúng crate `bamboo-core` (Rust v0.5.3) trực tiếp vào `fcitx5-lilypad/bamboo/`.
  - [x] Cấu hình CMake tự động build thư viện tĩnh Rust `libbamboo_core.a` (Custom Target `cargo build --release`).
- [x] **Phase 6.2: Viết Lớp C-ABI FFI Wrapper (`src/fcitx.rs` & `include/bamboo-core.h`):**
  - [x] Bọc các hàm C-ABI `NewEngine`, `NewCustomEngine`, `EngineProcessKeyEvent`, `EnginePullCommit`, `EnginePullPreedit`, `EngineCommitPreedit`, `ResetEngine`, `EngineRebuildFromText`, `GetInputMethodNames`, `GetCharsetNames`, `DeleteObject`.
  - [x] Quản lý đối tượng an toàn qua atomic handle table (`OBJECTS`), triệt tiêu 100% rủi ro use-after-free/crash.
- [x] **Phase 6.3: Tích hợp với Sequencer Layer & Kiểm Thử Hồi Quy:**
  - [x] Tích hợp liên kết `Bamboo::Core` vào `liblilypad.so`.
  - [x] Viết unit/integration test C++ kiểm tra toàn diện Telex (`tieengs` $\to$ `tiếng`), Space commit, Backspace undo (`tôi` $\to$ `tô`), và rebuild từ text (`chiêu` + `f` $\to$ `chiều`).
- [x] **Phase 6.4: Cập Nhật Trình Đóng Gói (All Distros & CI):**
  - [x] Cập nhật PKGBUILD AUR và `.SRCINFO` (đổi `makedepends` từ `go` sang `rust`).
  - [x] Cập nhật RPM Spec cho Fedora ([fcitx5-lilypad.spec](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/packaging/rpm/fedora/fcitx5-lilypad.spec)) và openSUSE ([fcitx5-lilypad.spec](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/packaging/rpm/opensuse/fcitx5-lilypad.spec)) sang `BuildRequires: cargo`, `BuildRequires: rust`.
  - [x] Cập nhật Debian package ([debian/control](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/packaging/debian/control)) sang `Build-Depends: cargo, rustc`.
  - [x] Cập nhật GitHub Workflows CI (`build.yml`, `packaging-release.yml`, `packaging-dev.yml`).
  - [x] Loại bỏ hoàn toàn mã nguồn Go cũ trong `fcitx5-lilypad/bamboo/` để repo sạch sẽ 100%.

---

### 🎯 Lộ Trình Triển Khai Trọng Tâm (Phase 8 — Kiến Trúc Sequencer Toàn Diện Cho Rich Text & Heavy DOM):
> **Mục tiêu:** Giải quyết triệt để lỗi gõ tiếng Việt trên TikTok Web (khung bình luận 25K+ comments, khung caption video), Lexical Editor và các trình soạn thảo Rich Text nặng mà không cần sao chép cơ chế Select của Lotus:
> 1. Triệt tiêu hiện tượng nảy tiêu điểm ảo khiến bộ đệm bị xóa oan (`no` + `s` $\to$ `nos`). *(Đã xong ở 8.1)*
> 2. Triệt tiêu hiện tượng nuốt ký tự commit & vỡ từ khi có dấu (`cháu` $\to$ `ch`, `couo7`). *(Mục tiêu 8.2)*
> 3. Tự động dãn Settling Window theo độ nghẽn của cây Virtual DOM khổng lồ (25K comments). *(Mục tiêu 8.3)*
> 4. Khai tử rủi ro xóa lẹm của phím Sentinel $N+1$ bằng Hardware Barrier Key vô hại (`KEY_PAUSE`). *(Mục tiêu 8.4)*

- [x] **Phase 8.1: Cơ Chế Chống Nảy Tiêu Điểm (Focus Bounce Protection / 100ms Deferral):**
  - [x] Thêm biến cờ `deferred_focus_out_ = false` và mốc `last_deactivate_time_ = 0` vào `LilypadState`.
  - [x] Khi nhận sự kiện `InputContextFocusOut` ở chế độ Uinput:
    - Không gọi `clearAllBuffers()` ngay lập tức.
    - Đặt cờ `deferred_focus_out_ = true` và ghi `last_deactivate_time_ = now_ms()`.
  - [x] Khi nhận sự kiện `InputContextFocusIn`:
    - Tính $\Delta t = \text{now\_ms}() - \text{last\_deactivate\_time\_}$.
    - Nếu $\Delta t < 100\text{ms}$ (Focus Bounce của Chromium/TikTok): Hủy bỏ nợ xóa (`cancelDeferredFocusOut()`), giữ nguyên vẹn `oldPreBuffer_` và Caret Buffer trong RAM.
    - Nếu $\Delta t \ge 100\text{ms}$ (Người dùng chuyển ứng dụng thật): Thực thi xả nợ xóa (`flush_deferred_focus_out()`), làm sạch bộ đệm an toàn.
  - [x] Đảo ngược điều kiện lỗi `< 100` thành `>= 100` tại `activate()`. Nâng phiên bản lên `v2.3.7`.

- [x] **Phase 8.2: Sequencer Settling Delay & Khóa Chặt Hàng Đợi (Atomic Commit Lock) (Hoàn tất - v2.3.8):**
  - [x] Thêm khoảng nghỉ lắng đọng `settling_delay_us = 6000 ~ 8000` ($7\text{ms}$ context-aware floor) vào Sequencer giữa nhịp phát phím Backspace cuối cùng và nhịp phát lệnh Commit.
  - [x] Bổ sung trạng thái `BarrierState::SettlingDelay` và phương thức `Sequencer::set_settling_delay()`.
  - [x] Khóa cứng hàng đợi Sequencer (`is_deleting_ / lock`) trong toàn bộ chu trình Xóa $\to$ Settle $\to$ Commit để bảo đảm tính nguyên tử (Atomic Transaction), bọc an toàn `if (!focusBounce)` tại `activate()` và loại bỏ ngắt cờ xóa sớm.
  - [x] Cho phép luồng xử lý sự kiện của ứng dụng (DOM reconciliation / Hashtag & Mention parser của TikTok) ổn định vị trí con trỏ trước khi ký tự commit mới đáp xuống, triệt tiêu lỗi nuốt commit `cháu` $\to$ `ch`.

- [x] **Phase 8.3: Bộ Nhận Diện Nghẽn DOM Động & Tự Động Điều Hòa Hai Chiều Cho Sequencer Layer (Per-Tab Dynamic Heavy-DOM & Event-Loop Governor) (Hoàn tất triển khai mã nguồn - v2.3.9):**
  - **Bối cảnh & Vấn đề thực tế (Bài học từ v2.3.8):**
    - Mức sàn cố định $7\text{ms}$ Pre-Commit Settling Delay kết hợp $70\text{ms}$ Post-Commit Settling Window trên Chromium khiến các tab nhẹ (Facebook, Google Docs trống) bị giam phím tổng cộng $\approx 80\text{ms}$, gây cảm giác chữ nhả ra chậm.
    - Ngược lại, trên các tab nặng (TikTok 25K bình luận, AFFiNE Canvas BlockSuite, Antigravity 2.0 DOM lớn), $7\text{ms}$ vẫn không đủ khi React DOM hoặc Editor re-render mất $30\text{ms} \sim 60\text{ms}$.
    - Không thể phân biệt bằng `appName` vì cả Facebook, TikTok, AFFiNE Web đều chạy trong cùng tiến trình `google-chrome`. Do đó, cần cơ chế đo tải động theo từng tab thời gian thực (Per-Tab Real-time Adaptive Control).
  - **4 Tín Hiệu Cơ Học Nhận Diện Tải Tại Runtime (Per-Context Mechanical Signals):**
    1. **Tín hiệu 1 — Measured App ACK Latency (`last_measured_ack_ms_`):**
       - Phản ánh trực tiếp tốc độ tiêu hóa phím của tab hiện tại.
       - Tab nhẹ (Facebook/Docs): $\text{ACK} \le 15\text{ms} \sim 20\text{ms}$.
       - Tab nặng (TikTok 25K cmt, AFFiNE): $\text{ACK}$ vọt lên $40\text{ms} \sim 70\text{ms}$ (thực tế đo được $60\text{ms}$).
    2. **Tín hiệu 2 — Áp Lực Hàng Đợi Phím Đệm (`buffered_keys_.size()` & `KeyEntry::enter_time`):**
       - `buffered_keys_.empty()` $\implies$ App bắt kịp tốc độ ngón tay, không nghẽn luồng.
       - `buffered_keys_.size() \ge 1` và phím bị giam $\ge 25\text{ms} \sim 30\text{ms}$ $\implies$ Event Loop/DOM của app đang bị dồn ứ, chậm hơn tốc độ ngón tay.
    3. **Tín hiệu 3 — Tốc Độ Vòng Lặp Nuốt Phím (`swallow_duration_us`):**
       - Thời gian từ lúc uinput phát chuỗi xóa đến khi Fcitx5 nuốt xong toàn bộ Backspace dội về.
    4. **Tín hiệu 4 — Tần Suất Nảy Tiêu Điểm Ảo (Focus Bounce Occurrences):**
       - Đánh dấu các editor có cơ chế DOM reconciliation giật cục (TikTok, Slate, Lexical).
  - **Cơ Chế Điều Hòa Đồng Bộ 2 Mắt Xích Sequencer:**
    - **Mắt xích 1: Pre-Commit Settling Delay (`settling_delay_us`) trong Sequencer:**
      - *Tab Mượt (Zero-Latency Mode - Facebook, Google Docs, Terminal):* Tự động co về mức cực tiểu $1.8\text{ms} \sim 2\text{ms}$ (hoặc $1.5\text{ms}$ cho non-Chromium). Chữ xuất hiện tức thì, triệt tiêu hoàn toàn cảm giác nhả chữ chậm.
      - *Tab Lag / Heavy-DOM (Adaptive Pacing Mode - TikTok, AFFiNE, Antigravity):* Tự động dãn nở theo công thức thích ứng:
        $$\text{settling\_delay\_us} = \text{clamp}(\text{last\_swallow\_duration} \times 0.6, 8\text{ms}, 35\text{ms})$$
        Nếu hàng đợi có phím chờ (`buffered_keys_.size() \ge 1`), tự động cộng thêm buffer margin $+4\text{ms} \sim 15\text{ms}$ để bảo vệ tuyệt đối vị trí con trỏ DOM.
    - **Mắt xích 2: Post-Commit Settling Window (`settle_timer_` / `is_deleting_` hold duration):**
      - *Tab Mượt:* Tự động co từ $70\text{ms}$ xuống $12\text{ms}$, giải phóng hàng đợi phím ngay khi ký tự commit đã an tọa (nhanh gấp 6 lần).
      - *Tab Lag:* Dãn nở động lên $60\text{ms} \sim 95\text{ms}$ tương ứng với độ trễ DOM, bảo vệ vòng lặp `deleteSurroundingText` không nuốt phím tiếp theo.
    - **Cơ Chế Snapback Tức Thì (Zero-Latency Snapback):**
      - Ngay khi hàng đợi phím được xả sạch (`buffered_keys_.empty()`) và nhịp ACK/swallow trở về mức bình thường ($\le 12\text{ms}$), toàn bộ hệ số trễ lập tức hạ về mức tối thiểu, không gây trễ dư thừa cho các từ tiếp theo.
  - **Các Bước Triển Khai Cụ Thể (Sub-tasks):**
    - [x] **Phase 8.3.1:** Thêm nhãn thời gian `enter_time` cho `KeyEntry` và trường theo dõi tải động trong `LilypadState` (`last_swallow_duration_us_`, `last_focus_bounce_time_`, `has_recent_focus_bounce_`).
    - [x] **Phase 8.3.2:** Xây dựng các hàm tính toán trễ động hai chiều: `isTabLagging()`, `calculateDynamicSettlingDelayUs()` và `calculateDynamicPostSettlingUs()`.
    - [x] **Phase 8.3.3:** Liên kết bộ tính trễ động vào `Sequencer::set_settling_delay()` và callback của `commit_timer_` / `settle_timer_` trong `lilypad-state.cpp`.
    - [x] **Phase 8.3.4:** Triển khai cơ chế Snapback tức thì giải phóng ngay độ trễ khi hàng đợi rỗng.
    - [x] **Phase 8.3.5:** Khắc phục lỗi nuốt chữ Rich-Text & Thay thế kép liên tiếp (v2.3.11 - v2.3.13):
      * Khôi phục điều tốc cảm biến IKI `micro_delay_us` (tự dãn $33\text{ms} \sim 35\text{ms}$ khi gõ thong thả) kết hợp sàn phần cứng $16\text{ms}$ ($16000\mu\text{s}$ - 1 khung hình 60Hz) trên Chromium (v2.3.12 - sửa dứt điểm lỗi nuốt chữ `có` $\to$ `c`, `bạn` $\to$ `b`).
      * Khôi phục mức sàn an toàn $70\text{ms}$ cho Post-Commit Settling Window trên Chromium (bảo vệ 100% các từ thay thế kép liên tiếp `thấy`, `thương`).
      * Nâng trần an toàn Hard Timeout Watchdog & Sequencer Max ACK từ $250\text{ms}$ lên $300\text{ms}$ (v2.3.13 - chống false alarm khi Chromium V8 GC lag trên web nặng).

- [x] **Phase 8.4: Kiến Trúc Hardware Barrier Key `KEY_F24` & Barrier Safety Timer (Giải Pháp Toàn Diện Triệt Để):**
  - **Mục tiêu cốt lõi:** Khai tử hoàn toàn phím Sentinel Backspace thứ $N+1$ (vốn là phím xóa có tính hủy hoại), chuyển sang dùng phím mở rộng phần cứng **`KEY_F24`** vô hại kết hợp **Barrier Safety Timer (20ms)**.
  - **Lợi ích kiến trúc đột phá:**
    1. *Bóc tách 100% tín hiệu đo ACK khỏi phím xóa:* Uinput chỉ bắn đúng $N$ phím Backspace (để xóa text cũ) và chốt đuôi bằng 1 phím `KEY_F24`.
    2. *Tuyệt đối không bao giờ xóa nhầm chữ:* Vì `KEY_F24` là phím vô hại, không có bất kỳ app hay trình nghe nhạc/game nào gán phím F24. Kể cả khi có lag hay lọt xuống app thì app cũng hoàn toàn phớt lờ.
    3. *Không bao giờ bị đơ 300ms:* Nếu Compositor làm rơi packet `KEY_F24` do Focus Bounce của TikTok/AFFiNE, Barrier Safety Timer (20ms) tự động kích hoạt xả rào chắn và commit chữ ngay lập tức.
  - **Các bước triển khai cụ thể:**
    - [x] **Phase 8.4.1 (Kernel Uinput Server Daemon):**
      * Trong `fcitx5-lilypad/server/lilypad-server.cpp`: Mở thêm capability `ioctl(fd, UI_SET_KEYBIT, KEY_F24)` khi khởi tạo uinput device.
      * Thêm hàm `send_f24()` phát cặp sự kiện nguyên tử `KEY_F24` (Press + Release + SYN_REPORT).
      * Cập nhật vòng lặp uinput: Sau khi bắn sạch $N$ phím `KEY_BACKSPACE`, tự động phát 1 phím chốt `KEY_F24`.
    - [x] **Phase 8.4.2 (Addon Sequencer & State Core):**
      * Trong `lilypad-state.cpp`: Khi phát lệnh thay thế (`performReplacement`), đặt `expected_backspaces_ = bsCount` (chỉ đúng $N$ phím xóa).
      * Trong `handleUInputKeyPress`: Bắt sự kiện phím `KEY_F24` (`event.key().check(FcitxKey_F24)`), nuốt trọn (`event.filterAndAccept()`), đo thời gian thực tế `swallow_duration_us`, giải phóng rào chắn và chuyển sang chu kỳ Settling Delay $\to$ Commit.
    - [x] **Phase 8.4.3 (Tích Hợp Barrier Safety Timer 20ms):**
      * Cài đặt timer an toàn $20\text{ms}$ ngay khi bắn uinput. Nếu phím `KEY_F24` bị Compositor làm rơi do Focus Bounce $\implies$ Timer tự động giải phóng rào chắn và commit ngay lập tức, triệt tiêu thời gian chờ vô ích $300\text{ms}$ của Watchdog.
    - [x] **Phase 8.4.4 (Biên Dịch & Cập Nhật Systemd Service):**
      * Biên dịch cả `fcitx5-lilypad` và daemon `fcitx5-lilypad-server` thành công 100% cả bản Debug và Release (v2.3.14).
      * Khởi động lại service và reload `fcitx5`.

- [x] **Phase 8.5: Focus Settled Debounce Cho Chromium (v2.4.6):**
  - [x] Thiết lập bộ đếm lắng dịu 35ms tại `LilypadEngine::activate()` khi có Focus Bounce trong lúc xóa vi mô.
  - [x] Tự động đo đạc telemetry thực tế $t_{\text{end}} - t_{\text{uinput\_send}}$ nạp vào Dynamic Governor, triệt tiêu lỗi kẹt 300ms Watchdog.
  - [x] Bảo vệ an toàn phím Space và hàng đợi phím bấm qua Batch Replay Protocol 3ms.

- [x] **Phase 8.6: Chống Rụng Phím Vật Lý Do Wayland Focus Bounce (v2.4.7 - v2.4.11 Hoàn Thành):**
  - **Tài liệu đặc tả:** Xem chi tiết 4 giải pháp tại [wayland-focus-bounce-drop-key-solutions.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/1-overview/project-managers/wayland-focus-bounce-drop-key-solutions.md).
  - [x] **Giải pháp 1:** Lưới bảo hiểm phần cứng từ `fcitx5-lilypad-server` (lắng nghe phím bàn phím qua `libinput` trên `seat0` độc lập với Niri Compositor - v2.4.7).
  - [x] **Giải pháp 2:** Cổng lọc trùng phím hai chiều (Bi-directional Keystroke Dedup Gate - v2.4.10).
  - [x] **Giải pháp 3:** Grace Delimiter Engine Reset chống dính từ (v2.4.11).

- [x] **Phase 8.7: Khóa Hàng Đợi FIFO Tuần Tự & Single FIFO Batch Commit (v2.4.12 - v2.4.13 Hoàn Thành):**
  - [x] **FIFO Queue Lock:** Trong `keyEvent()`, nếu `!buffered_keys_.empty()` hoặc `replay_timer_` đang chạy, đẩy mọi phím Niri mới vào cuối hàng đợi `buffered_keys_` để triệt tiêu lỗi phím mới vượt mặt làm lọt Space vào giữa từ (`sách tr ời`).
  - [x] **Server Watchdog Expansion:** Nâng `TAP_WATCHDOG_TIMEOUT_MS` từ 120ms lên 300ms trong `lilypad-server.cpp` tương thích hoàn toàn với chu kỳ DOM của Chrome/AFFiNE.
  - [x] **Consolidated Batch Commit Pipeline:** Gom toàn bộ phím chờ trong RAM xả 1 lần qua `ic_->commitString()`, loại bỏ giật lag DOM.

- [x] **Phase 8.8: Dynamic Settling Floor 20ms VSync & SetMode App Sync (v2.4.14 - v2.4.16 Hoàn Thành):**
  - [x] **Lotus-Aligned 20ms VSync Floor:** Chuẩn hóa Pre-commit Settling Delay lên $18\text{ms} \sim 22\text{ms}$ (1 VSync frame $16.6\text{ms} +$ headroom an toàn) cho Chromium/Electron và $5\text{ms}$ Post-commit settling window cho hàng đợi RAM. Triệt tiêu hoàn toàn lỗi bung phím VNI `co6 thuo7g` trên AFFiNE Canvas.
  - [x] **SetMode Application State Sync:** Tự động đồng bộ `wa_chromium_flag = true` khi người dùng chuyển mode sang Sequence, sửa dứt điểm lỗi mất chữ `a -> ""` trên Antigravity IDE.
  - [x] **Multi-tier Wayland App Resolution:** Tra cứu process 3 tầng trực tiếp qua Niri IPC Socket ($155\mu\text{s}$) khi `ic->program()` rỗng.

- [x] **Phase 9: Major Milestone Release v2.5.0, Cross-Distro Packaging & AUR Publication (Hoàn Thành):**
  - [x] Đóng gói phát hành phiên bản lớn `v2.5.0`, cập nhật `CHANGELOG.md`.
  - [x] Viết lại toàn bộ `README.md` theo phong cách giới thiệu sản phẩm (Product Showcase).
  - [x] Chuẩn hóa danh mục dependencies cho Arch Linux, Ubuntu / Debian và Fedora.
  - [x] Đóng gói file nhị phân `fcitx5-lilypad-v2.5.0-x86_64-archlinux.tar.zst` (1.6MB).
  - [x] Tạo tag `v2.5.0` và push lên GitHub `origin`.
  - [x] Xuất bản đồng bộ 3 gói AUR: `fcitx5-lilypad`, `fcitx5-lilypad-bin`, `fcitx5-lilypad-git`.

