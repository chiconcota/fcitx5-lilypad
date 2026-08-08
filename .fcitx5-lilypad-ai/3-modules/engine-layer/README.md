# MODULE: ENGINE LAYER (`fcitx5-lilypad/src/lilypad-engine.h/.cpp` & `lilypad-state.h/.cpp`)

@status: STABLE (v2.2.0-modular-sensor) | @last_update: 2026-08-07

---

## 1. TỔNG QUAN VÀ VAI TRÒ KẾT NỐI (OVERVIEW)

**Engine Layer** (`LilypadEngine` & `LilypadState`) quản lý máy trạng thái xử lý tiếng Việt cho dự án **`fcitx5-lilypad`** (`vnlilypad-lotus`), kết nối trực tiếp với thư viện lõi **Bamboo Engine (Go C-FFI `bamboo-core`)**.

> **Triết lý thiết kế:** Tận dụng 100% sức mạnh xử lý quy tắc Telex/VNI từ Bamboo Engine (thư viện Go được đóng gói qua C-FFI `CGoObject`), kết hợp quản lý bộ đệm trạng thái `LilypadState` theo từng ngữ cảnh nhập liệu (`InputContext`).

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

### 3. `bamboo-core` (Go C-FFI Telex Engine)
- Nhận phím gõ thô, thực hiện kiểm tra chính tả và biến đổi từ tiếng Việt.
- Tự động khôi phục từ Tiếng Anh (English Non-VN Restore) khi phát hiện từ không chuẩn tiếng Việt.

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
 │             LilypadState :: processKey / Bamboo C-FFI                  │
 ├────────────────────────────────────────────────────────────────────────┤
 │ - Process Telex/VNI transformations via bamboo-core C-FFI              │
 │ - Compute minimal deleted suffix (compareAndSplitStrings)              │
 │ - Send micro-steps to LilypadSequencer / Uinput Server                 │
 └────────────────────────────────────────────────────────────────────────┘
```


