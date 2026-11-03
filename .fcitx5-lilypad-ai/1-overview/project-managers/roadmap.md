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

### 🛡️ Kế Hoạch Tương Lai (Phase 7 — Cơ Chế Nhận Diện Nghẽn DOM Động & Tự Động Điều Hòa $T_{\text{render}}$):
> **Mục tiêu:** Nhận diện chính xác 100% khi nào ứng dụng bị nghẽn DOM (như Antigravity 134k nodes) mà không phụ thuộc vào chính tả người gõ, không gán cứng $60\text{ms} \sim 80\text{ms}$, bảo toàn tốc độ bàn thờ $1\text{ms}$ cho Terminal và cứu chữ tuyệt đối cho Webview nặng.

- [ ] **Phase 7.1: Gắn Nhãn Mốc Thời Gian Cho Phím Đệm (`KeyEntry::enter_time`):**
  - [ ] Thêm trường `std::chrono::steady_clock::time_point enter_time` vào cấu trúc `KeyEntry` trong `lilypad-state.h`.
  - [ ] Ghi mốc thời gian ngay khi phím bị đưa vào hàng đợi `buffered_keys_.push_back()`.
- [ ] **Phase 7.2: Triển Khai Bộ Nhận Diện Lag Bằng Tuổi Thọ Phím & Áp Lực Hàng Đợi (Dwell Time & Queue Pressure):**
  - [ ] Bật cờ `is_app_lagging = true` khi thỏa mãn một trong các điều kiện cơ học:
    1. **Tuổi thọ phím bị giam:** $\text{Age} = \text{now} - \text{enter\_time} \ge 40\text{ms} \sim 50\text{ms}$ (phím quá cảnh lâu bất thường).
    2. **Thời gian rào chắn bị giam giữ:** $\Delta t_{\text{locked}} = \text{now} - uinput\_send\_time\_ \ge 40\text{ms}$.
    3. **Độ sâu tích tụ hàng đợi:** `buffered_keys_.size() >= 2` (dồn ứ nhiều hơn 1 phím).
    4. **Gõ chậm mà vẫn bị nghẽn buffer:** `(is_deleting_ && current_iki >= 50ms)`.
- [ ] **Phase 7.3: Tự Động Điều Hòa $T_{\text{render}}$ & Post-Commit Settling Window Động:**
  - [ ] Khi `is_app_lagging == true`: Tự động nở $T_{\text{render}}$ và Settling Window lên mức an toàn ($70\text{ms} \sim 100\text{ms}$) cho đến khi xả sạch hàng đợi.
  - [ ] Khi `is_app_lagging == false` (Terminal, App mượt, hoặc khi App đã hạ nhiệt): Co $T_{\text{render}}$ về $0\text{ms}$, trả lại tốc độ tức thì $1\text{ms} \sim 2\text{ms}$.
- [ ] **Phase 7.4: Kiểm thử thực nghiệm đối chứng:**
  - [ ] Kiểm thử gõ siêu tốc trên Terminal/Kitty: Xác nhận hàng đợi không bị phình, $T_{\text{render}} = 0\text{ms}$.
  - [ ] Kiểm thử trên Antigravity đoạn chat nặng: Xác nhận phát hiện nghẽn tức thì, không rụng chữ (`nhìn`, `quá`).

---

### 🎯 Kế Hoạch Trọng Tâm Phiên Tới (Phase 8 — Chống Focus Bounce & Sequencer Settling Atomic Commit):
> **Mục tiêu:** Giải quyết triệt để lỗi gõ tiếng Việt trên TikTok Web và các trình soạn thảo Rich Text nặng (React/Lexical/Hashtag Editor) mà không cần sao chép cơ chế Select của Lotus:
> 1. Triệt tiêu hiện tượng nảy tiêu điểm ảo khiến bộ đệm bị xóa oan (`no` + `s` $\to$ `nos`).
> 2. Triệt tiêu hiện tượng nuốt ký tự commit do đụng độ bộ phân tích cú pháp ngầm (`cháu` $\to$ `ch`).

- [ ] **Phase 8.1: Cơ Chế Chống Nảy Tiêu Điểm (Focus Bounce Protection / 100ms Deferral):**
  - [ ] Thêm biến cờ `deferred_focus_out_ = false` và mốc `last_deactivate_time_ = 0` vào `LilypadState`.
  - [ ] Khi nhận sự kiện `InputContextFocusOut` ở chế độ Uinput:
    - Không gọi `clearAllBuffers()` ngay lập tức.
    - Đặt cờ `deferred_focus_out_ = true` và ghi `last_deactivate_time_ = now_ms()`.
  - [ ] Khi nhận sự kiện `InputContextFocusIn`:
    - Tính $\Delta t = \text{now\_ms}() - \text{last\_deactivate\_time\_}$.
    - Nếu $\Delta t < 100\text{ms}$ (Focus Bounce của Chromium/TikTok): Hủy bỏ nợ xóa (`deferred_focus_out_ = false`), giữ nguyên vẹn `oldPreBuffer_` và Caret Buffer trong RAM.
    - Nếu $\Delta t \ge 100\text{ms}$ (Người dùng chuyển ứng dụng thật): Thực thi xả nợ xóa (`flush_deferred_focus_out()`), làm sạch bộ đệm an toàn.
- [ ] **Phase 8.2: Sequencer Settling Delay & Khóa Chặt Hàng Đợi (Atomic Commit Lock):**
  - [ ] Thêm khoảng nghỉ lắng đọng `settling_delay_us = 6000 ~ 8000` ($6\text{ms} \sim 8\text{ms}$) vào Sequencer giữa nhịp phát phím Backspace cuối cùng và nhịp phát lệnh Commit.
  - [ ] Khóa cứng hàng đợi Sequencer (`is_deleting_ / lock`) trong toàn bộ chu trình Xóa $\to$ Settle $\to$ Commit để bảo đảm tính nguyên tử (Atomic Transaction).
  - [ ] Cho phép luồng xử lý sự kiện của ứng dụng (DOM reconciliation / Hashtag & Mention parser) ổn định vị trí con trỏ trước khi ký tự commit mới đáp xuống.
- [ ] **Phase 8.3: Thay Thế Sentinel Backspace (N+1) Bằng Hardware Barrier Key:**
  - [ ] Khai tử phím Backspace thứ $N+1$ (nguồn gốc gây xóa lẹm vào lệnh Terminal và văn bản web).
  - [ ] Sử dụng `KEY_PAUSE` (hoặc phím chức năng vô hại) làm rào cản đồng bộ phần cứng ở đuôi chuỗi Backspace.
  - [ ] Fcitx5 chặn `KEY_PAUSE`, kích hoạt Commit và xả hàng đợi phím an toàn.
- [ ] **Phase 8.4: Kiểm Thử Thực Địa & Xác Nhận:**
  - [ ] Kiểm thử trên TikTok Web (Khung bình luận): Gõ `no` + `s` $\to$ ra đúng `nó`, không bị rơi rụng thành `nos`.
  - [ ] Kiểm thử trên TikTok Web (Khung đăng video caption): Gõ `cháu` $\to$ ra đúng `cháu`, không bị nuốt mất thành `ch`.
  - [ ] Kiểm thử trên Terminal/CLI: Xác nhận không sinh mã rác và không xóa lẹm dòng lệnh.

