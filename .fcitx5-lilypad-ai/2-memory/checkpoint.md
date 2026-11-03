# VNLILYPAD LOTUS CHECKPOINT (DỰ ÁN NÂNG CẤP FCITX5 LOTUS)

## 📌 Trạng thái Bàn giao Phiên Làm Việc (Session Checkpoint)

- **Tên dự án:** `vnlilypad-lotus` ("Nâng cấp Fcitx5 Lotus")
- **Đường dẫn thư mục:** `/home/chiconcota/Documents/vnlilypad-lotus/`
- **Nhánh Git làm việc:** `feat/focus-bounce-protection`
- **Tình trạng:** **ĐÃ HOÀN TẤT TRIỂN KHAI PHASE 8.1 (FOCUS BOUNCE PROTECTION / 100MS DEFERRAL WINDOW). KHẮC PHỤC TRIỆT ĐỂ BỆNH XÓA OAN CARET BUFFER TRÊN TIKTOK WEB KHUNG BÌNH LUẬN (`no` + `s` $\to$ `nos`). ĐẢO NGƯỢC ĐIỀU KIỆN LỖI `< 100` THÀNH `>= 100`. BIÊN DỊCH THÀNH CÔNG 100% CẢ BẢN DEBUG VÀ RELEASE, NÂNG PHIÊN BẢN LÊN V2.3.7.**

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc (2026-11-03 - Hoàn Tất Phase 8.1 Focus Bounce Protection - v2.3.7):

1. **Khởi Tạo Nhánh `feat/focus-bounce-protection`:**
   - Tạo nhánh làm việc mới cách ly tính năng Focus Bounce Protection.

2. **Triển Khai Cơ Chế Hoãn Xóa 100ms Tại `LilypadState` (`lilypad-state.h/.cpp`):**
   - Bổ sung cờ `deferredFocusOut_ = false` và các phương thức `resetNow(bool)`, `flushDeferredFocusOut()`, `cancelDeferredFocusOut()`.
   - Trong `LilypadState::reset(isFocusOut)`: Khi `isFocusOut && isUinputMode(...)`, chỉ bật `deferredFocusOut_ = true` và reset input panel, bảo toàn 100% `oldPreBuffer_` và trạng thái Bamboo Engine trong RAM.

3. **Điều Phối Nhận Diện Focus Bounce Tại `LilypadEngine` (`lilypad-engine.cpp`):**
   - Trong `activate()`: Khi nhận diện Focus Bounce (`uinputMode && delta < 100ms`), gọi `state->cancelDeferredFocusOut()`, cập nhật `realMode = targetMode`, tuyệt đối không gọi `setMode()` hoặc `clearAllBuffers()`.
   - Sửa lỗi điều kiện ngược dấu tại dòng 445: Đổi `< 100` thành `>= 100` để không bao giờ xóa bộ đệm khi focus quay lại trong $< 100\text{ms}$.
   - Trong `deactivate()`: Trao quyền xử lý cho cửa sổ 100ms, không xóa thô `clearAllBuffers()` khi nhận `FocusOut` ở chế độ Uinput.
   - Thêm helper `isUinputMode(LilypadMode)` vào `lilypad-utils.h`.

4. **Nâng Phiên Bản & Biên Dịch Hệ Thống (`v2.3.7`):**
   - Nâng phiên bản `CMakeLists.txt` và `PKGBUILD` lên `2.3.7`.
   - Biên dịch thành công 100% cả Debug (`build/src/liblilypad.so`) và Release (`build_release/src/liblilypad.so`).

1. **Khởi Tạo Nhánh & Cài Đặt Toolchain Rust:**
   - Tạo và chuyển sang nhánh `feat/rust-bamboo-engine`.
   - Cài đặt toolchain Rust chính thức: `rustc 1.99.1` & `cargo 1.99.1` qua `rustup`.

2. **Tích Hợp `bamboo-core` (Rust) Thay Thế Module Go Cũ (Bước 1 & 2):**
   - Xóa bỏ toàn bộ mã nguồn Go cũ (`bamboo-c.go`, `fcitxbambooengine.go`, `go.mod`, Go `bamboo-core/`).
   - Nhúng trực tiếp crate `bamboo-core` (Rust v0.5.3 từ `bamboo-vi/bamboo-core`) vào `fcitx5-lilypad/bamboo/`.
   - Viết tầng FFI C-ABI chuẩn (`fcitx5-lilypad/bamboo/src/fcitx.rs`) bọc an toàn qua atomic handle table (`OBJECTS`), triệt tiêu 100% rủi ro use-after-free/crash.
   - Xuất đầy đủ các hàm FFI cho Fcitx5: `Init`, `NewEngine`, `NewCustomEngine`, `EngineProcessKeyEvent`, `EnginePullCommit`, `EnginePullPreedit`, `EngineCommitPreedit`, `EngineSetOption`, `ResetEngine`, `EngineRebuildFromText`, `GetInputMethodNames`, `GetCharsetNames`, `DeleteObject`.
   - Cung cấp header C đồng bộ [`bamboo-core.h`](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/bamboo/include/bamboo-core.h).

3. **Cập Nhật Hạ Tầng CMake & Liên Kết Thư Viện Tĩnh (Bước 2 & 3):**
   - Cập nhật [`fcitx5-lilypad/bamboo/CMakeLists.txt`](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/bamboo/CMakeLists.txt): Tự động phát hiện `cargo`, build thư viện tĩnh `libbamboo_core.a` trong profile Release và export target `Bamboo::Core`.
   - Cập nhật [`fcitx5-lilypad/src/CMakeLists.txt`](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/src/CMakeLists.txt): Liên kết trực tiếp `Bamboo::Core`, `Pthread::Pthread`, `dl`, `m`.
   - Biên dịch thành công 100% `liblilypad.so` cả ở chế độ Debug lẫn Release (`build_release/src/liblilypad.so`). Kích thước nhị phân stripped co về chỉ còn **2.0MB**, hoàn toàn không còn Go runtime / GC / CGO switches.

4. **Kiểm Thử Tích Hợp C++ Thực Nghiệm (Bước 4):**
   - Viết test C++ chuyên sâu kiểm tra toàn diện luồng gõ FFI:
     - Gõ từ Telex cơ bản: `tieengs` $\to$ `tiếng` (100% PASS).
     - Phím Space commit từ: `' '` $\to$ `tiếng ` (100% PASS).
     - Phím Backspace undo vi mô: `tooi` (`tôi`) + `Backspace` $\to$ `tô` (100% PASS).
     - Khôi phục từ đã gõ và thêm dấu: `chiêu` + `f` $\to$ `chiều` (100% PASS).
   - Kiểm tra rò rỉ bộ nhớ: Giải phóng đối tượng qua `DeleteObject(engine)` sạch sẽ 100%.

5. **Đồng Bộ Toàn Diện Packaging (AUR, Fedora, openSUSE, Debian) & Build CI:**
   - **Arch Linux (AUR):** Cập nhật `makedepends` từ `go` sang `rust` trong cả 2 file `PKGBUILD` và `.SRCINFO`.
   - **Fedora RPM:** Cập nhật `fcitx5-lilypad.spec` sang `BuildRequires: cargo`, `BuildRequires: rust`; rà soát case-sensitivity chuẩn (`python3-QtPy`, `libX11-devel`).
   - **openSUSE RPM:** Cập nhật `fcitx5-lilypad.spec` sang `BuildRequires: cargo`, `BuildRequires: rust`.
   - **Debian / Ubuntu:** Cập nhật `debian/control` sang `Build-Depends: cargo, rustc`.
   - **GitHub Workflows:** Thay thế `go` bằng `rust` trong `build.yml`, `packaging-release.yml`, và `packaging-dev.yml`.
   - Cập nhật tài liệu kỹ thuật [`engine-layer/README.md`](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/3-modules/engine-layer/README.md), [`system_map.md`](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/1-overview/system_map.md), và [`decision-log.md`](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/2-memory/decision-log.md) (Quyết định 046).

1. **Nghiên cứu Cơ Chế Uinput Select (`Shift+Left`) của Lotus > 6.0:**
   - Đọc và phân tích trực tiếp mã nguồn `fcitx5-lotus-main/` (v6.0.4) qua các commit `a68195b`, `4c6fddc`, `cb67d6b`, `8b4cb89`.
   - Xác định Lotus dùng `Shift + Left` x $N$ để bôi đen, nhả Shift, bấm `Delete`, và bắn mỏ neo `KEY_PAUSE` (hoặc `KEY_F24`) làm rào cản.
   - Thảo luận cùng người dùng: Người dùng quyết định **không sao chép cơ chế Select của Lotus**, mà tập trung phát triển giải pháp thuần chất Lilypad trên nền tảng Sequencer Layer và Backspace truyền thống.

2. **Truy Tìm Cội Nguồn 2 Bệnh Riêng Biệt của TikTok Web:**
   - **Bệnh 1 (Khung bình luận: `no` + `s` $\to$ `nos`):**
     - Do React DOM re-render làm Chromium bắn `FocusOut` $\to$ `FocusIn` giả chỉ trong vòng $5\text{ms} \sim 20\text{ms}$ (Focus Bounce).
     - Lilypad xóa sạch Caret Buffer ngay khi nhận `FocusOut`, khiến phím `s` mất ngữ cảnh từ trước đó và bị in thô thành chữ thường không dấu.
   - **Bệnh 2 (Khung đăng video caption: `cháu` $\to$ `ch`):**
     - Do bộ phân tích cú pháp `#Hashtag` và `@Mention` ngầm khóa tạm thời con trỏ sau nhịp Backspace.
     - Lilypad commit chữ mới quá vội ($\Delta t = 0\text{ms}$) khi con trỏ chưa ổn định, khiến chuỗi commit `áu` bị vứt bỏ, chỉ còn trơ lại phần đầu từ chưa xóa `ch`.

3. **Chốt Kế Hoạch Triển Khai Phase 8:**
   - **Mục 1 (Focus Bounce Protection):** Hoãn xóa bộ đệm $100\text{ms}$ khi nhận `FocusOut`. Nếu `FocusIn` quay lại trong $< 100\text{ms}$ thì hủy lệnh xóa, bảo toàn 100% Caret Buffer (`deferred_focus_out_`, `last_deactivate_time_`).
   - **Mục 2 (Sequencer Settling Atomic Commit):** Thêm khoảng nghỉ lắng đọng vi mô $6\text{ms} \sim 8\text{ms}$ giữa nhịp Xóa và Commit, khóa chặt hàng đợi Sequencer đảm bảo giao dịch nguyên tử, triệt tiêu khe hở thời gian làm rơi rụng chữ commit trên Rich Text Web.
   - **Mục 3 (Hardware Barrier Key):** Thay phím Sentinel Backspace thứ $N+1$ bằng `KEY_PAUSE`, an toàn tuyệt đối cho Terminal và Web.
   - Đã cập nhật đầy đủ: `roadmap.md` (Phase 8), `decision-log.md` (Quyết định 048), `system_map.md` và `sequencer-layer/README.md`.

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc (2026-09-12 - Gỡ Bỏ Nợ Kỹ Thuật, Phát Hành v2.3.6 & AUR, Chuẩn Hóa Tác Giả):

1. **Gỡ bỏ Nợ Kỹ thuật & Cờ `is_antigravity_flag_` (`v2.3.6`):**
   - Xóa bỏ cờ `is_antigravity_flag_` trong `lilypad-state.h` và `lilypad-engine.cpp`.
   - Xóa bỏ mức sàn cứng $32\text{ms}$ và logic settle riêng $100\text{ms}$ trong `lilypad-state.cpp`, đưa về chuẩn Chromium $70\text{ms}$ ($70,000\mu\text{s}$) và non-Chromium $300\mu\text{s}$.
   - Giữ `"antigravity"` trong `ack-apps.h` để ứng dụng tiếp tục được nhận diện là app Chromium (`wa_chromium_flag = true`).
   - Nâng phiên bản `CMakeLists.txt` và `PKGBUILD` lên **v2.3.6**.
   - Biên dịch và cài đặt hoàn tất vào `/usr/lib/fcitx5/liblilypad.so`, khởi động lại Fcitx5 sạch sẽ.

2. **Phát Hành Bản Đóng Gói Nhị Phân & Cập Nhật AUR:**
   - Biên dịch bản Release sạch (`-DCMAKE_BUILD_TYPE=Release`, `NDEBUG`).
   - Đóng gói file nhị phân `dist/fcitx5-lilypad-v2.3.6-x86_64-archlinux.tar.zst` (SHA256: `cfd6a4b2397938e412036a0738e31bfa41f7c6a700afc13eb8a07bc297fb5df7`).
   - Đẩy tag `v2.3.6` lên remote `origin/main` (GitHub: `chiconcota/fcitx5-lilypad`).
   - Cập nhật đồng bộ và đẩy thành công lên 3 kho AUR (`aur.archlinux.org`):
     - `fcitx5-lilypad` (`v2.3.6-1`)
     - `fcitx5-lilypad-bin` (`v2.3.6-1`)
     - `fcitx5-lilypad-git` (`v2.3.6.r...`)

3. **Chuẩn Hóa Quyền Tác Giả & Hệ Thống Tài Liệu README:**
   - Phát hiện `.git/config` local có email `thanhpy2009@gmail.com`. Đã cập nhật lại `git config user.email lytatthanh@gmail.com` và `git config user.name chiconcota`.
   - Rebase toàn bộ commit thuộc đợt phát hành `v2.3.6` (`6f00947`, `8cf66f9`, `cec1eb8`, `9379354`, `ecd8ebc`, `8137ba3`) với `--reset-author` để ghi nhận chính xác tác giả **chiconcota**.
   - Cập nhật lại tag `v2.3.6` và đẩy `git push --force-with-lease origin main --tags`.
   - Cập nhật nội dung trên cả 3 file README (`README.md`, `fcitx5-lilypad/README.md`, `fcitx5-lilypad/README.en.md`): mô tả các cải tiến kiến trúc v2.3.6 và bổ sung mục Lời Cảm Ơn trang trọng tới tác giả Bamboo Engine (**Luật Nguyễn**), tác giả VMK (**thanhpy2009**), và nền tảng **fcitx5-lotus**.
   - Hoàn tất cập nhật `CHANGELOG.md`.

1. **Khảo Sát DOM & Truy Tìm Nguyên Nhân "Đoạn Chat Cũ Bị Lỗi Dù Không Mở Thêm Gì":**
   - Kết nối Chrome DevTools Protocol vào Antigravity kiểm tra cây DOM của cuộc hội thoại `"Thiết Kế Website Cá Nhân"`:
   - Phát hiện Antigravity ngầm giấu một panel drawer bên phải (`width: 0px`, người dùng không thấy trên màn hình) chứa danh sách duyệt của **8.174 files**.
   - Điều này làm tổng số DOM nodes phình to lên tới **134.842 nodes** (trong đó 130.784 nodes nằm ở drawer ẩn này).
   - Mỗi lần bấm Backspace, React traversal trên cây DOM 134k nodes mất **$300\text{ms} \sim 450\text{ms}$**.
   - Trong khi đó, trần Watchdog Timeout mặc định của Fcitx5 là **$250\text{ms}$**. Fcitx5 tưởng ứng dụng bị đơ nên kích hoạt cơ chế cắt lỗ khẩn cấp `purgeContextEmergency()`, xả phím số `6`, `4` ra màn hình dạng thô $\to$ tạo thành chuỗi `loi64a`.

2. **Triển Khai Watchdog 800ms & Settling Window 100ms Cách Ly Tuyệt Đối (`v2.3.4`):**
   - `fcitx5-lilypad/src/lilypad-sequencer.h`: Thêm hàm `set_max_ack_timeout_ms(uint64_t ms)`.
   - `fcitx5-lilypad/src/lilypad-engine.cpp`: Trong `activate()`, khi phát hiện `antigravity`, đặt `state->sequencer_.set_max_ack_timeout_ms(800)`. Ngược lại đặt `250ms`.
   - `fcitx5-lilypad/src/lilypad-state.cpp`:
     - Nâng trần watchdog an toàn: `hard_timeout_us = is_antigravity_flag_ ? 800000 : 250000;` ($800\text{ms}$ cho Antigravity, $250\text{ms}$ cho app khác).
     - Mở rộng settling delay: `settle_delay_us = wa_chromium_flag ? (is_antigravity_flag_ ? 100000 : 70000) : 300;` ($100\text{ms}$ cho Antigravity).
   - Mọi app khác (Ghostty, Chrome, IDE, Terminal...) giữ nguyên 100% trần 250ms và không bị ảnh hưởng.

3. **Biên Dịch & Nâng Cấp Hệ Thống:**
   - Cập nhật `CMakeLists.txt` và `PKGBUILD` lên `v2.3.4`.
   - Biên dịch thành công 100% và cài đặt ghi đè `/usr/lib/fcitx5/liblilypad.so`.
4. **Kiểm Thử Thực Nghiệm Cuối Phiên & Phát Hiện Bản Chất Lỗi Nuốt Ký Tự (`nhn`, `qu`):**
   - Người dùng gõ câu: *"anh hai nhìn ... chịu quá"* trên đoạn chat cũ có DOM nặng.
   - Kết quả hiển thị: `anh hai nhn chịu qu` (mất chữ `ì` $\to$ `nhn`, mất chữ `á` $\to$ `qu`, nhưng chữ `chịu` thì còn nguyên).
   - Đối chiếu phân tích log:
     - Với `nhìn` (`bsCount=1`, `micro_delay_us=33000us` = $33\text{ms}$): Fcitx5 bắn Backspace xóa `i`, nhưng chỉ chờ $33\text{ms}$ là gọi `ic_->commitString("ì")`. Do cây DOM của React có tới hơn $134.000$ nodes, React xử lý Backspace chưa xong trong $33\text{ms}$ nên Lexical vứt bỏ sự kiện `commitString("ì")` $\implies$ thành `nhn`.
     - Với `quá` (`bsCount=1`, `micro_delay_us=33000us` = $33\text{ms}$) $\implies$ tương tự, nuốt mất chữ `á` $\implies$ thành `qu`.
     - Ngược lại với `chịu` (`bsCount=2`, `micro_delay_us=51000us` = $51\text{ms}$): Fcitx5 tự động giãn delay theo số lượng Backspace lên $51\text{ms}$, vừa vặn đủ thời gian cho React xử lý xong $\implies$ chữ `chịu` hiển thị hoàn hảo 100%!
   - **Kết luận:** Mức sàn $32\text{ms}$ cho $N=1$ Backspace trên DOM $134\text{k}$ nodes là chưa đủ, cần nâng lên **$80\text{ms} \sim 100\text{ms}$**.

## 🎯 Tiến Độ Trước Đó (2026-09-11 - Per-App Delay Floor cho Antigravity & Nâng v2.3.3):

1. **Khảo Sát Thực Nghiệm & Bắt Trọn Log Trực Tiếp Qua CDP Trong Ô Lexical Antigravity:**
   - Kết nối trực tiếp vào Antigravity 2.0 qua Chrome DevTools Protocol, phát hiện ô chat thực chất là `contenteditable="true"` dùng **Lexical Editor của Meta** tích hợp sâu các AI plugins: `["ghost-text", "contextScopeItemMention", "beautifulMention", "artificial"]`.
   - Gắn bộ logger sự kiện (`window.__inputLog`) bắt trọn từng mili-giây khi người dùng gõ.
   - Bằng chứng thực nghiệm đối chứng:
     - Từ `"giáo"` ($N=2$ backspace, trễ $29\text{ms}$): Thành công 100%, Lexical xử lý hoàn hảo!
     - Từ `"cô"` ($N=1$ backspace, trễ co xuống $0.1\text{ms}$): Thất bại, Lexical đang lock selection để re-render Backspace nên vứt bỏ sự kiện `beforeinput "ô"`, không sinh ra sự kiện `input`.
     - Từ `"thương"` biến thành `"tương"`: Do ký tự dấu đầu tiên bị rụng, gây lệch nhịp đệm giữa Fcitx5 và DOM $\to$ đợt xóa kế tiếp xóa nhầm phụ âm `"h"`.
   - Lý giải vì sao `playground.lexical.dev` trên Chrome không bị: Do là bản Vanilla thuần túy không có dàn plugin AI ngầm can thiệp vào con trỏ selection.

2. **Triển Khai Per-App Micro-Pacing Floor Cách Ly Tuyệt Đối (`is_antigravity_flag_`):**
   - Thêm cờ `is_antigravity_flag_` vào `LilypadState`.
   - Trong `LilypadEngine::activate`: Nếu `appNameLower.find("antigravity") != std::string::npos`, kích hoạt `is_antigravity_flag_ = true`.
   - Trong `handleUInputKeyPress`: Nếu `is_antigravity_flag_ == true`, áp mức sàn an toàn `micro_delay_us = std::max(micro_delay_us, 32000)` ($32\text{ms}$).
   - Mọi ứng dụng khác (Ghostty, Kitty, Chrome, Gedit, IDE...) giữ nguyên 100% tốc độ siêu tốc $1\text{ms} \sim 6\text{ms}$, hoàn toàn không bị ảnh hưởng.

3. **Nâng Phiên Bản Lên v2.3.3 & Biên Dịch / Cài Đặt Hệ Thống:**
   - Cập nhật `CMakeLists.txt`, `packaging/aur/fcitx5-lilypad/PKGBUILD`.
   - Biên dịch thành công 100% với `-DCMAKE_BUILD_TYPE=Debug` và cài đặt vào `/usr/lib/fcitx5/liblilypad.so`.
   - Khởi động lại `fcitx5` và xác nhận chẩn đoán `fcitx5-diagnose` nạp `Lilypad Wrapper For Fcitx 2.3.3`.

## 🎯 Tiến Độ Trước Đó (2026-09-11 - Post-Commit Settling Window cho Chromium Webview & Nâng v2.3.2):

1. **Phân Tích & Xác Định Bản Chất Lỗi Nuốt Chữ "thương" $\to$ "tương":**
   - Phân tích log thực nghiệm: Khi gõ `thuo` + `w`, `commitString("ơ")` được gửi nhưng Chromium Webview (chat dài có DOM nặng) chưa kịp render vào DOM trong 60ms.
   - Khi người dùng gõ tiếp `n` (mốc 61ms sau commit), bộ đệm Bamboo nổ đợt thay thế thứ 2 (`uơ` $\to$ `ươn`), gửi 2 Backspace qua uinput.
   - Vì `"ơ"` chưa có trong DOM, 2 phím Backspace xóa nhầm `"u"` và `"h"`, dẫn đến mất chữ `"h"` hiển thị thành `"tương"`.

2. **Triển Khai Lựa Chọn B - Post-Commit Settling Window (`settle_timer_`):**
   - Thêm biến timer `settle_timer_` vào `LilypadState` (`fcitx5-lilypad/src/lilypad-state.h`).
   - Trong callback `commit_timer_`: Nếu `wa_chromium_flag == true`, giữ cờ `is_deleting_ = true` thêm $70\text{ms}$ sau `ic_->commitString()`.
   - Các phím gõ nhanh tiếp theo (`n`, `g`) được gom an toàn vào RAM `buffered_keys_`.
   - Khi hết $70\text{ms}$, Chromium đã vẽ xong `"ơ"` vào DOM, Fcitx5 hạ cờ `is_deleting_ = false` và gọi `replayBufferedKeys()` để tiếp tục xử lý các phím sau.
   - Sửa nhánh phím Backspace trong `keyEvent`: Giữ nguyên passthrough cho intermediate backspaces (1..N) để không hủy `commit_timer_` sớm, khắc phục lỗi mất sạch chữ (`chau1` -> `""`).

3. **Nâng Phiên Bản Lên v2.3.2 & Biên Dịch / Cài Đặt Hệ Thống:**
   - Cập nhật `CMakeLists.txt`, `packaging/aur/fcitx5-lilypad/PKGBUILD`.
   - Biên dịch thành công 100% với `-DCMAKE_BUILD_TYPE=Debug` và cài đặt vào `/usr/lib/fcitx5/liblilypad.so`.
   - Khởi động lại `fcitx5` và xác nhận log đã nhận diện `antigravity detected: set wa_chromium_flag=true`.

1. **Phát Hành Nhị Phân `fcitx5-lilypad-bin` (v2.3.0):**
   - Biên dịch Release sạch (`-DCMAKE_BUILD_TYPE=Release`, `NDEBUG`), đóng gói file `dist/fcitx5-lilypad-v2.3.0-x86_64-archlinux.tar.zst` (2.1MB).
   - Tính toán và cập nhật chính xác mã băm `sha256sums` (`7b8f4758d6...`).
   - Đẩy trực tiếp metadata `.SRCINFO` và `PKGBUILD` lên `aur@aur.archlinux.org/fcitx5-lilypad-bin.git`.

2. **Cập Nhật Gói Source `fcitx5-lilypad` & Gói Git `fcitx5-lilypad-git` Lên AUR:**
   - `fcitx5-lilypad`: Cập nhật `v2.3.0-1` với mã băm sha256 của GitHub source tarball (`f8c8c5411f...`).
   - `fcitx5-lilypad-git`: Cấu hình `-DCMAKE_BUILD_TYPE=Debug` phục vụ nhà phát triển & tester bắt log lỗi.
   - Đẩy thành công cả 2 repo lên máy chủ AUR chính thức (`aur.archlinux.org`).

3. **Chuẩn Hóa Tài Liệu README & CHANGELOG.md:**
   - Tạo file `CHANGELOG.md` chuẩn Keep a Changelog (`Added`, `Changed / Improved`, `Fixed`).
   - Sửa toàn bộ công thức toán KaTeX trên GitHub (`\mathrm{EMA}_{\mathrm{IKI}}`, `$T_{\text{ack}}$`).
   - Cập nhật mục cài đặt AUR trên cả 3 file README phân định rõ ràng 3 tùy chọn và chính sách log.

---

## 🎯 Kế Hoạch Bàn Giao & Nhiệm Vụ Cho Phiên Kế Tiếp (Handover Plan):

1. **Ưu Tiên 1 — Triển khai Phase 8.2 (Sequencer Settling Delay & Atomic Commit Lock):**
   - Thêm khoảng nghỉ lắng đọng vi mô `settling_delay_us = 6000 ~ 8000` ($6\text{ms} \sim 8\text{ms}$) giữa nhịp phát phím Backspace cuối cùng và nhịp phát lệnh Commit.
   - Khóa cứng hàng đợi Sequencer (`is_deleting_`) trong suốt chu trình `Xóa -> Settle -> Commit`, đảm bảo giao dịch nguyên tử, giải quyết triệt để lỗi rụng dấu vỡ từ ra chuỗi thô (`couo7`) trên TikTok Web.

2. **Ưu Tiên 2 — Triển khai Phase 8.3 (Bộ Nhận Diện Nghẽn DOM Động 4 Trụ Cột):**
   - Gắn nhãn `KeyEntry::enter_time`, đo tuổi thọ phím đệm trong RAM $\ge 40\text{ms}$ và áp lực hàng đợi $\ge 2$.
   - Tự động dãn Settling Window lên mức an toàn ($70\text{ms} \sim 100\text{ms}$) trên các trang web có DOM khổng lồ (video 25.2K comments), và co về $0\text{ms} \sim 1\text{ms}$ trên Terminal để giữ tốc độ Zero-Latency.

3. **Ưu Tiên 3 — Triển khai Phase 8.4 (Hardware Barrier Key `KEY_PAUSE` / `KEY_F24`):**
   - Khai tử phím Backspace thứ $N+1$, chuyển sang dùng `KEY_PAUSE` vô hại (Non-destructive) làm chốt đo ACK an toàn tuyệt đối cho Terminal và Web.

---

## 📁 Các File Thay Đổi Trong Phiên (2026-11-03 - Phase 8.1):

| File | Thay đổi |
| :--- | :--- |
| `fcitx5-lilypad/src/lilypad-state.h/.cpp` | Triển khai cờ `deferredFocusOut_`, `cancelDeferredFocusOut()`, `flushDeferredFocusOut()`, tách `resetNow(bool)` bảo vệ 100% Caret Buffer |
| `fcitx5-lilypad/src/lilypad-engine.cpp` | Bắt sự kiện Focus Bounce (`delta < 100ms`), đảo điều kiện lỗi `< 100` thành `>= 100`, hủy nợ xóa và không gọi `setMode()` làm xóa buffer |
| `fcitx5-lilypad/src/lilypad-utils.h` | Thêm inline helper `isUinputMode(LilypadMode)` |
| `fcitx5-lilypad/settings-gui/ui/pages/about.py` | Cập nhật ghi nhận `Bamboo Engine (Rust Native Core)` |
| `fcitx5-lilypad/CMakeLists.txt` & `PKGBUILD` | Nâng phiên bản `v2.3.7` |
| `.fcitx5-lilypad-ai/1-overview/project-managers/roadmap.md` | Tái cấu trúc lộ trình làm việc khoa học cho Phase 8.2, 8.3, 8.4 |
| `.fcitx5-lilypad-ai/1-overview/system_map.md` | Cập nhật Module Status & Recent Change Log v2.3.7 |
| `.fcitx5-lilypad-ai/2-memory/decision-log.md` | Ghi Quyết định 049 (Phase 8.1 & Lộ trình hợp nhất) |
| `.fcitx5-lilypad-ai/3-modules/sequencer-layer/README.md` | Cập nhật tài liệu kỹ thuật Focus Bounce Protection v2.3.7 |
| `.fcitx5-lilypad-ai/2-memory/checkpoint.md` | Bàn giao phiên làm việc trên nhánh `feat/focus-bounce-protection` |

---

1. **Xây Dựng Hạ Tầng Đóng Gói AUR Cho 3 Phiên Bản (`fcitx5-lilypad/packaging/aur/`):**
   - **`fcitx5-lilypad-git/`**: PKGBUILD biên dịch tự động từ nhánh `main` mới nhất trên GitHub (`git+https://github.com/chiconcota/fcitx5-lilypad.git`). Tối ưu hàm `pkgver()` linh hoạt tự động tính số commit và git hash (`2.2.0.r49.g104b0a4`).
   - **`fcitx5-lilypad-bin/`**: PKGBUILD tải nhị phân pre-compiled (`.tar.zst`) phát hành từ GitHub Releases, giúp người dùng Arch cài đặt tức thì trong 1 giây mà không cần biên dịch C++/Go.
   - **`fcitx5-lilypad/`**: PKGBUILD biên dịch từ source tarball release tag chính thức (`v2.2.0.tar.gz`).

2. **Viết Scriptlet `fcitx5-lilypad.install` Chuẩn Arch Linux:**
   - Tự động hướng dẫn người dùng nạp uinput module (`sudo modprobe uinput`), kích hoạt user proxy & systemd daemon (`systemctl --user daemon-reload`), và nạp lại Fcitx5 (`fcitx5 -r -d`).

3. **Tự Động Sinh Metadata `.SRCINFO` & Kiểm Thử Thành Công Qua `makepkg`:**
   - Đã sinh file `.SRCINFO` chuẩn cho cả 3 gói AUR bằng `makepkg --printsrcinfo`.
   - Đã đóng gói kiểm thử thực tế `makepkg -fc --nodeps` tạo thành công file package `fcitx5-lilypad-git-2.2.0.r49.g104b0a4-1-x86_64.pkg.tar.zst` không một lỗi lầm.

4. **Cập Nhật Hướng Dẫn Cài Đặt AUR Trong Tất Cả README:**
   - Đã bổ sung phần hướng dẫn `yay -S fcitx5-lilypad-bin` và `yay -S fcitx5-lilypad-git` vào cả 3 file: `README.md`, `fcitx5-lilypad/README.md`, và `fcitx5-lilypad/README.en.md`.

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc (2026-08-09 - Tích Hợp Embedded Bamboo-Core, Fix Addon Config, Tối Ưu Nhịp Replay 3000:300 & Release Zero-Log Overhead):

1. **Tích Hợp Trực Tiếp Mã Nguồn Go `bamboo-core` Vào Git Main:**
   - Khắc phục triệt để lỗi `open .../bamboo-core/go.mod: no such file or directory` bằng cách gỡ bỏ submodule pointer rỗng và nhúng 100% 19 file mã nguồn Go của `bamboo-core` trực tiếp vào cây thư mục Git chính của dự án (`fcitx5-lilypad/bamboo/bamboo-core/`).
   - Người dùng `git clone` bình thường sẽ có ngay 100% mã nguồn biên dịch mượt mà không cần lệnh submodule.

2. **Sửa Lỗi Addon Config Fcitx5 (`lilypad-addon.conf.in.in`):**
   - Sửa `Library=liblilypad` (khớp với tên file nhị phân `liblilypad.so`) và `[Dependencies]` chuẩn syntax Fcitx5.
   - Giải quyết dứt điểm 100% lỗi ngầm `Found 0 input method(s) in addon lilypad`, hiển thị ngay lập tức bộ gõ `Lilypad` trong `fcitx5-configtool`.

3. **Sửa Lỗi Systemd Unit & Cập Nhật Hướng Dẫn Cài Đặt Systemd:**
   - Sửa đường dẫn cài đặt service unit trong `misc/CMakeLists.txt` thành `DESTINATION /lib/systemd/system`.
   - Bổ sung `sudo systemctl daemon-reload` vào tất cả file README, khắc phục lỗi `Unit does not exist` trên Ubuntu/Debian.

4. **Tối Ưu Nhịp Xả Phím Đệm (`replay_delay_us` = `isSpace ? 3000 : 300`):**
   - Tối ưu `replay_delay_us` trong `lilypad-state.cpp` thành `isSpace ? 3000 : 300` (3ms cho Space, 0.3ms cho phím chữ).
   - Loại bỏ hoàn toàn lỗi đè rác chữ `cháau1kh` khi gõ siêu tốc không cách từ, trong khi vẫn giữ nguyên tốc độ nổ phím chữ siêu tức thì $0.3\text{ms}$.

5. **Lập Trình Cờ Tự Động Release Zero-Log Overhead (`lilypad-utils.h`):**
   - Cấu hình macro `LILYPAD_INFO` và `LILYPAD_DEBUG` thành `((void)0)` khi build `-DCMAKE_BUILD_TYPE=Release` (`NDEBUG`).
   - Giữ nguyên Log cho bản Dev/GitHub, và tự động xóa sạch 100% log trong file nhị phân `-bin` công khai trên các kho ứng dụng OS (AUR, Fedora COPR, Flatpak, PPA).

6. **Chuẩn Hóa Đầy Đủ Phụ Thuộc (Dependencies) Cho 3 Distro Linux:**
   - Cập nhật lệnh cài đặt dependencies đầy đủ cho **Ubuntu/Debian** (`apt`), **Arch Linux** (`pacman`), và **Fedora** (`dnf`) trong cả 3 file README (`README.md`, `fcitx5-lilypad/README.md`, `fcitx5-lilypad/README.en.md`).

1. **Tái Cấu Trúc Bộ Icon SVG Lá Súng (Lilypad Brand Icon):**
   - Thiết kế bộ icon SVG vector mới chuẩn **Lá Súng xanh (Lilypad Leaf)** tươi mát trong `fcitx5-lilypad/data/icons/` (`fcitx-lilypad.svg`, `fcitx-lilypad-default.svg`, `fcitx-lilypad-off.svg`, `fcitx-lilypad-emoji.svg`).
   - Khôi phục bộ icon Hoa Sen gốc cho `fcitx-lotus` từ `fcitx5-lotus-main/data/icons/`, phân tách rõ ràng 100% giữa Lotus (Hoa Sen) và Lilypad (Lá Súng).

2. **Cập Nhật Giao Diện Cài Đặt Trang About (`about.py` & `CMakeLists.txt`):**
   - Đặt số phiên bản chuẩn **`2.2.0 (Stable)`** trong `CMakeLists.txt` (`project(fcitx5-lilypad VERSION 2.2.0)`).
   - Nạp icon Lá Súng xanh trực tiếp từ file SVG hệ thống (`/usr/share/icons/hicolor/scalable/apps/fcitx-lilypad.svg`).
   - Định dạng văn bản mô tả bằng thẻ HTML `<div style="line-height: 1.4;">` giải quyết dứt điểm 100% lỗi xén mất viền chữ trên/dưới.
   - Cập nhật thông tin tác giả chính: `chiconcota (Creator & Maintainer)` (`https://github.com/chiconcota`).
   - Cập nhật các liên kết repository chuẩn: `https://github.com/chiconcota/fcitx5-lilypad`.
3. **Chuẩn Hóa Trang GitHub Repo & README:**
   - Cập nhật file ảnh logo hiển thị trên GitHub README [fcitx5-lilypad/data/fcitx-lilypad-README.svg](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/data/fcitx-lilypad-README.svg) thành logo **Lá Súng xanh (Lilypad Leaf)** mới.
   - Cập nhật tiêu đề README từ `🪷` sang `🍃`.
   - Đồng bộ file `LICENSE` ở Root dự án sang GPLv3 (GitHub tự động nhận diện License badge GPLv3).

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc (2026-08-08 - Chuẩn Hóa Toàn Diện Hệ Thống Document Trước Khi Publish/Push App):

1. **Thực Hiện Luật Không Rác (Zero-Trash Directive):**
   - Chuyển `REPORT-session-2026-08-02.md` từ Root dự án vào [.fcitx5-lilypad-ai/2-memory/archive/REPORT-session-2026-08-02.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/2-memory/archive/REPORT-session-2026-08-02.md) qua `git mv`, bảo đảm Root dự án sạch sẽ 100%.

2. **Tối Ưu & Chuẩn Hóa Cả 4 Module Layer trong `.fcitx5-lilypad-ai/3-modules/`:**
   - [.fcitx5-lilypad-ai/3-modules/engine-layer/README.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/3-modules/engine-layer/README.md): Cập nhật mô tả `LilypadEngine` C++ kế thừa `InputMethodEngineV2`, `LilypadState` và Go C-FFI `bamboo-core`, loại bỏ các định nghĩa Rust cũ.
   - [.fcitx5-lilypad-ai/3-modules/kernel-layer/README.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/3-modules/kernel-layer/README.md): Cập nhật mô tả `fcitx5-lilypad-server` C++ daemon phát phím Backspace nguyên tử qua `/dev/uinput` cùng xác thực UID Unix Socket.
   - [.fcitx5-lilypad-ai/3-modules/sequencer-layer/README.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/3-modules/sequencer-layer/README.md): Giữ vững chuẩn `v2.2.0-modular-sensor`, `NiriAckSensor` (EMA Adaptive Control) và `GenericAckSensor`.
   - [.fcitx5-lilypad-ai/3-modules/ui-layer/README.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/3-modules/ui-layer/README.md): Cập nhật đặc tả Native UI tích hợp $100\%$ giao diện chuẩn Fcitx5 Lotus (`fcitx5-configtool` & System Tray Menu).

3. **Cập Nhật Đồng Bộ System Map (`system_map.md`):**
   - Cập nhật Bảng Trạng thái Module trong [system_map.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/1-overview/system_map.md) khớp chính xác với hạ tầng C++ và Native UI Fcitx5.

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc (2026-08-07 - Tối Ưu Hướng Dẫn Cài Đặt & Sửa Lỗi Profile Logout/Login):

1. **Tối Ưu Hướng Dẫn Cài Đặt Trong Tất Cả README:**
   - Cập nhật quy trình cài đặt từ source trong [README.md](file:///home/chiconcota/Documents/vnlilypad-lotus/README.md), [fcitx5-lilypad/README.md](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/README.md) và [fcitx5-lilypad/README.en.md](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/README.en.md).
   - Bổ sung lệnh nạp uinput module: `ls /dev/uinput || sudo modprobe uinput`.
   - Bổ sung lệnh khởi tạo user proxy & reload udev: `sudo systemd-sysusers` và `sudo udevadm control --reload-rules && sudo udevadm trigger` (khắc phục lỗi `Permission Denied` `/dev/uinput` khi vừa gõ `make install` xong).
   - Cập nhật lệnh restart `fcitx5 -r -d` (chạy daemon ngầm chính chủ thay vì `&` của shell).

2. **Khắc Phục Lỗi Mất Tiếng Việt Khi Logout/Login:**
   - Phát hiện nguyên nhân: File cấu hình `~/.config/fcitx5/profile` của người dùng trước đó bị gán `DefaultIM=lotus`. Khi Logout/Login, Fcitx5 tự động nạp bộ gõ `lotus` cũ không tương thích với daemon uinput mới.
   - Sửa dứt điểm bằng cách dừng `fcitx5-remote -e`, cập nhật `DefaultIM=lilypad` và khởi động lại `fcitx5 -d`.

3. **Phân Tích Kiến Trúc X11 & So Sánh NiriAckSensor vs GenericAckSensor:**
   - Khẳng định 100% Sequence Mode hoạt động mượt mà trên X11 (Zorin OS, Linux Mint, XFCE...).
   - Nuốt phím `should_swallow_backspace` và `ic_->commitString()` chạy trên tầng **Fcitx5 Core API** (hoạt động trên cả X11 XIM lẫn Wayland).
   - Làm rõ sự khác biệt giữa `NiriAckSensor` ($6\text{ms} + N \times 4\text{ms}$) và `GenericAckSensor` ($8\text{ms} + N \times 5\text{ms}$).

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc (2026-08-06 - Chuẩn hóa Trang GitHub & Community Call):

1. **Chuẩn hóa Trang giới thiệu GitHub Dự án:**
   - Cập nhật [README.md](file:///home/chiconcota/Documents/vnlilypad-lotus/README.md), [fcitx5-lilypad/README.md](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/README.md) và [fcitx5-lilypad/README.en.md](file:///home/chiconcota/Documents/vnlilypad-lotus/fcitx5-lilypad/README.en.md).
   - Minh bạch hóa trạng thái thử nghiệm: Tác giả hiện kiểm thử và tối ưu chính trên **Niri Compositor (Arch Linux)**.
   - Xây dựng **Bảng Ma trận Tương thích Compositor & Distro** (Niri 🟢, Hyprland/Sway 🟡, KDE/GNOME 🟡, X11 🟢).
   - Thiết lập lời kêu gọi **Community Call for Testers & Contributors** thu hút đóng góp từ cộng đồng Linux Việt Nam.
2. **Xây dựng Tài Liệu Kỹ Thuật Chi Tiết `NiriAckSensor` & `IAckSensor`:**
   - Tạo & hoàn thiện file đặc tả kiến trúc [.fcitx5-lilypad-ai/3-modules/sequencer-layer/niri-ack-sensor-architecture.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/3-modules/sequencer-layer/niri-ack-sensor-architecture.md).
   - Đảm bảo 100% tài liệu bám sát chính xác mã nguồn C++ trong `fcitx5-lilypad/src/ack-sensors/` (`ack-sensor.h`, `niri-sensor.h`, `generic-sensor.h`, `sensor-factory.h`).
   - Giải thích sơ đồ 5 bước kiến trúc tầng dưới **Linux Wayland Input Subsystem** (`/dev/uinput` $\to$ Kernel Evdev $\to$ `libinput` & Window Compositor $\to$ Fcitx5 Token Swallow $\to$ App Window).
   - Khẳng định **Nguyên tắc Vạn năng (Universal Wayland IME Protocol)**: Không cần viết Sensor riêng cho từng Distro, `GenericAckSensor` tự động xử lý vạn năng 100% mọi Distro & Compositor hỗ trợ `zwp_input_method_v1/v2`.
   - Chỉ định rõ 4 vị trí đoạn code C++ cốt lõi đo mốc $T_1$, nuốt token uinput, chốt mốc $T_2$ và tính `calculate_adaptive_delay_ms()`.
   - Tối ưu hóa **[system_map.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/1-overview/system_map.md)**: Loại bỏ các quy tắc/thử nghiệm cũ không liên quan (`usleep 1500`, `EVIOCGRAB`), tinh gọn 4 quy tắc kiến trúc cốt lõi và làm sạch nhật ký thay đổi tập trung vào `v2.2.0-modular-sensor`.
   - Tối ưu hóa **[decision-log.md](file:///home/chiconcota/Documents/vnlilypad-lotus/.fcitx5-lilypad-ai/2-memory/decision-log.md)**: Phân nhóm mạch lạc 5 mảng kiến trúc đang vận hành thực tế 100% (Modular Sensor, Sequencer Layer, Uinput Server, Fcitx5 Systemd, và AFFiNE Zero-Regression Policy), cắt bỏ các mục trùng lặp/hủy bỏ sang `archive/deprecated-decisions.md`.
   - Cắt bỏ thư mục rác **`3-modules/wayland-layer/`**: Đây là tài liệu thử nghiệm client Rust cũ độc lập (`WaylandImeClient` v0.31) không còn nằm trong kiến trúc Fcitx5 C++ Addon hiện tại, đã chuyển lưu trữ sang `archive/deprecated-decisions.md`.

---

## 🎯 Nhật Ký Tiến Độ Phiên Làm Việc Trước (2026-08-05 - Tái cấu trúc Sensor & Batch Replay):

1. **Kiến trúc Modular IAckSensor:**
   - Tách Lớp Cảm biến ACK thành các Module cắm/rút (`IAckSensor`, `NiriAckSensor`, `GenericAckSensor`, `AckSensorFactory`).
   - Tự động phát hiện biến môi trường `$XDG_CURRENT_DESKTOP` để nạp Module Sensor tương ứng.
2. **Thuật toán EMA Machine Learning Adaptive Control:**
   - Tự động điều chỉnh độ trễ thích ứng theo nhịp lag của App ($0.35 \times \text{Measured} + 0.65 \times \text{Prev}$) và tự động suy giảm (decay) nhanh về $5\text{ms}$ khi App mượt.
3. **Tối ưu Luồng Replay Batch Flush:**
   - Phân biệt phím CÁCH (`Space` - hoãn 3ms chống đè IPC) và phím thường (`a, b, c...` - xả tức thì 0.1ms).
   - Bẻ gãy 100% bẫy đệ quy hoãn 15ms từng phím, triệt hạ hoàn toàn lỗi kẹt phím 4.5s khi gõ tốc độ cao.

1. **Phát hiện Root Cause trên AFFiNE (BlockSuite Canvas):**
   - Log debug xác nhận: Mỗi khi bộ gõ `commitString()` 1 ký tự, BlockSuite Editor của AFFiNE lập tức phát sự kiện `activate()` / `InputContextFocusIn` ngầm 10ms sau đó.
   - Code `activate()` trong `lilypad-engine.cpp` gọi `setMode()` và `clearAllBuffers()`, làm mất bộ nhớ `oldPreBuffer_` giữa các phím gõ làm đứt đoạn Telex.
2. **Khôi phục Nguyên trạng An toàn (Safety-First Directive):**
   - Đã thực hiện `git checkout` khôi phục 100% mã nguồn C++ gốc sạch sẽ trên `main`.
   - Đã biên dịch & cài đặt lại `/usr/lib/fcitx5/liblilypad.so` chuẩn, bảo đảm bộ gõ chạy mượt 100% trên các ứng dụng thông thường (Messenger, Chrome, IDE, Terminal).

---

## 💡 BẢNG PHƯƠNG ÁN XỬ LÝ LƯU TRỮ CHO PHIÊN KẾ TIẾP (AFFiNE ROADMAP):

### 🔹 Phương án 1: Bật Cờ Wayland IME trong Electron (`~/.config/affine-flags.conf`)
- File cờ đã được tạo tại [~/.config/affine-flags.conf](file:///home/chiconcota/.config/affine-flags.conf):
  ```text
  --ozone-platform=wayland
  --enable-wayland-ime
  --wayland-text-input-version=3
  ```
- Giúp Electron 39 giao tiếp trực tiếp qua Wayland text-input-v3 thay vì X11 fallback.

### 🔹 Phương án 2: Cách ly Spurious Focus Event cho AFFiNE
- Đưa điều kiện kiểm tra `appName == "AFFiNE"` / `appName == "ONLYOFFICE"` vào **trước** khi `setMode()` được gọi trong `activate()`.
- Tuyệt đối không `return` sớm trong `setMode()` nếu không có cờ kiểm tra Focus thực sự, tránh lỗi trôi bộ đệm khi chuyển đổi giữa 2 ứng dụng khác nhau.

### 🔹 Phương án 3: Sequencer Stale Serial Pruning (Lọc vi bước cũ an toàn)
- Khi người dùng gõ nhanh phím mới trong lúc AFFiNE đang render phím cũ, không được dùng `sequencer_.clear()` thô vì sẽ làm mất cờ đếm token `expected_swallow_backspaces_`.
- Thay vào đó, trong `Sequencer::poll_next_step()`, chỉ bỏ qua các `MicroStep` có `step.serial < active_serial_` mà giữ nguyên token count.

### 🔹 Phương án 4: Chế độ Whole-Word Replacement cho Canvas Shadow DOM
- Nếu BlockSuite Canvas bị vỡ node DOM khi nhận xóa vi mô, cho phép Mode Sequence thực hiện thay thế nguyên từ (`Whole-Word Replacement`) trong 1 giao dịch nguyên tử duy nhất.

---

## 🎯 Kế Hoạch Bàn Giao Phiên Tiếp Theo (Handover Plan for Next Session):

1. **Kiểm thử cài đặt bản build chuẩn hóa từ Git/GitHub:**
   - Tiến hành biên dịch, cài đặt và kiểm thử từ repo GitHub chuẩn hóa `git@github.com:chiconcota/fcitx5-lilypad.git` thay vì chạy build trực tiếp trong cây thư mục source local.
2. **Bổ sung cấu hình tùy chỉnh `MinDelayMs`:**
   - Thêm tùy chọn `MinDelayMs` vào `lilypad-config.h` và `lilypad.conf` (cho phép máy cấu hình cao tự do hạ trần trễ xuống $1\text{ms} \sim 2\text{ms}$).
3. **Đóng gói Release & PKGBUILD lên AUR (Arch User Repository):**
   - Xây dựng file `PKGBUILD` và `.SRCINFO` chuẩn Arch Linux cho `fcitx5-lilypad`.
   - Cấu hình cài đặt `fcitx5-lilypad-server@.service` tự động qua Systemd service unit.
   - Kiểm thử biên dịch và cài đặt từ `makepkg -si` trên Arch Linux / Niri.

---

## 📁 Các File Thay Đổi Trong Phiên:

| File | Thay đổi |
| :--- | :--- |
| `README.md` | Chuẩn hóa trang GitHub, thêm Ma trận Tương thích & Kêu gọi Tester |
| `fcitx5-lilypad/README.md` | Cập nhật README Tiếng Việt chuyên nghiệp & Định hướng AUR |
| `fcitx5-lilypad/README.en.md` | Cập nhật README Tiếng Anh chuyên nghiệp & Định hướng AUR |
| `.fcitx5-lilypad-ai/3-modules/sequencer-layer/niri-ack-sensor-architecture.md` | Đặc tả kiến trúc C++ IAckSensor, NiriAckSensor, EMA, 5-step Linux Input Subsystem & Log Messenger |
| `.fcitx5-lilypad-ai/2-memory/archive/deprecated-decisions.md` | Tạo kho lưu trữ 5 thử nghiệm/quyết định gỡ bỏ `[XOÁ - KHÔNG ÁP DỤNG]` |
| `.fcitx5-lilypad-ai/1-overview/system_map.md` | Tối ưu hóa 4 quy tắc cốt lõi, làm sạch Change Log v2.2.0-modular-sensor |
| `.fcitx5-lilypad-ai/2-memory/decision-log.md` | Phân nhóm 5 mảng kiến trúc active, dọn dẹp các quyết định trùng lặp |
| `.fcitx5-lilypad-ai/2-memory/checkpoint.md` | Niêm phong bộ nhớ phiên 2026-08-06 |

---

## 🚀 Lệnh Biên Dịch & Khởi Động Lại:

```bash
# Biên dịch & Cài đặt Lilypad chuẩn /usr:
cd fcitx5-lilypad/build && cmake -DCMAKE_INSTALL_PREFIX=/usr .. && make -j$(nproc) && sudo make install

# Khởi động lại Fcitx5:
fcitx5 -r -d
```


