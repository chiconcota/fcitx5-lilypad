# MODULE: ENGINE LAYER (`fcitx5-lilypad/src/lilypad-engine.h/.cpp` & `lilypad-state.h/.cpp`)

@status: PRODUCTION READY (v2.5.0-rust-bamboo-core) | @last_update: 2026-11-07

---

## 1. TỔNG QUAN VÀ VAI TRÒ KẾT NỐI (OVERVIEW)

**Engine Layer** (`LilypadEngine` & `LilypadState`) quản lý máy trạng thái xử lý tiếng Việt cho dự án **`fcitx5-lilypad`** (`vnlilypad-lotus`), kết nối trực tiếp với thư viện lõi **Bamboo Engine (Rust C-ABI `bamboo-core`)**.

> **Triết lý thiết kế:** Tận dụng 100% sức mạnh xử lý quy tắc Telex/VNI từ Bamboo Engine (thư viện Rust được đóng gói qua C-ABI `libbamboo_core.a`), kết hợp quản lý bộ đệm trạng thái `LilypadState` theo từng ngữ cảnh nhập liệu (`InputContext`). Không còn Go runtime, không còn GC, triệt tiêu 100% CGO context switch.

---

## 2. KIẾN TRÚC VÀ CÁC THÀNH PHẦN CỐT LÕI (CORE COMPONENTS)

### 1. `LilypadEngine` (`fcitx5-lilypad/src/lilypad-engine.h/.cpp`)
- Kế thừa `InputMethodEngineV2` từ **Fcitx5 Core Framework**.
- Quản lý vòng đời `activate()`, `deactivate()`, `keyEvent()`, `reset()` và chuyển đổi chế độ gõ (`Sequence`, `Smooth`...).
- Quản lý quy tắc bộ gõ theo từng ứng dụng (`appRules_`), load/save file cấu hình `lilypad.conf`.
- Tự động nạp từ điển macro (`lilypadMacroTable`) và biểu tượng Emoji qua `EmojiLoader`.

### 2. `LilypadState` (`fcitx5-lilypad/src/lilypad-state.h/.cpp`)
- Quản lý Virtual Caret Buffer (`oldPreBuffer_`) cho từng cửa sổ ứng dụng.
- Thực hiện so sánh chuỗi tối ưu `compareAndSplitStrings` để chỉ gửi phím xóa cho phần hậu tố tối thiểu (`deletedPart`).
- Điều phối luồng `performReplacement()` gửi lệnh xóa qua Kernel Uinput Server và bắn chuỗi chữ mới qua `commitString()`.

### 3. `bamboo-core` (Rust C-ABI Telex Engine)
- Nhận phím gõ thô, thực hiện kiểm tra âm tiết bằng Deterministic Finite Automaton (DFA) và biến đổi từ tiếng Việt.
- Tự động khôi phục từ Tiếng Anh (English Non-VN Restore) khi phát hiện từ không chuẩn tiếng Việt.
- Tích hợp sẵn cơ chế encode charset (Unicode, TCVN3, VNI Windows...).
- Tốc độ phản hồi cực nhanh (< 1µs / keystroke), tiêu thụ RAM < 1MB.

### 4. Giao diện FFI & Thư viện Tĩnh `Bamboo::Core`
- **Mã nguồn FFI:** `fcitx5-lilypad/bamboo/src/fcitx.rs` cung cấp 16 hàm FFI C-ABI chuẩn hóa quản lý qua bảng handle atomic `OBJECTS`.
- **C Header:** `fcitx5-lilypad/bamboo/include/bamboo-core.h` định nghĩa kiểu dữ liệu `bamboo_engine_t`, `bamboo_custom_engine_t`, `bamboo_key_event_t`.
- **CMake Target:** `fcitx5-lilypad/bamboo/CMakeLists.txt` tự động gọi Cargo biên dịch Release và xuất target `Bamboo::Core` (`libbamboo_core.a`) liên kết trực tiếp vào `liblilypad.so`.

---

## 3. GIAO TIẾP VỚI HỆ THỐNG (SYSTEM INTEGRATION)

```text
 ┌────────────────────────────────────────────────────────────────────────┐
 │                     FCITX5 INPUT CONTEXT EVENT                         │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │ (KeyEvent)
                                     ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │            LilypadEngine :: keyEvent (fcitx5-lilypad)                  │
 ├────────────────────────────────────────────────────────────────────────┤
 │ - Check App Rules & Current Typing Mode (Sequence / Smooth)            │
 │ - Pass Raw Key to LilypadState                                         │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │
                                     ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │            LilypadState :: processKey / Bamboo Rust C-ABI              │
 ├────────────────────────────────────────────────────────────────────────┤
 │ - Process Telex/VNI transformations via bamboo-core C-ABI              │
 │ - Compute minimal deleted suffix (compareAndSplitStrings)              │
 │ - Send micro-steps to LilypadSequencer / Uinput Server                 │
 └────────────────────────────────────────────────────────────────────────┘
```


