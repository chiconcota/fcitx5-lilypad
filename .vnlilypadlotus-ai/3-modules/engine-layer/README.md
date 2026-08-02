# MODULE: ENGINE LAYER (`src/engine/`)

@status: MILESTONE 1 (ENGLISH TEST ENGINE & ACK SEQUENCER VERIFIED) | @last_update: 2026-07-26

---

## 1. TỔNG QUAN VÀ VAI TRÒ KẾT NỐI (OVERVIEW)

**Engine Layer** (`TypingEngine`) chứa máy trạng thái thuần túy (Pure State Machine) biến đổi các phím gõ thô (raw keypresses) thành chuỗi hành động IME semantic (`ImeAction`).

> **Triết lý thiết kế:** Cách ly $100\%$ logic xử lý tiếng Việt với tầng I/O và hệ thống (Decoupled Engine). Engine không hề biết về uinput, Wayland hay IPC socket; nó chỉ nhận chuỗi phím vào và trả về danh sách `ImeAction`.

---

## 2. INTERFACE & IMPLEMENTATIONS (`src/engine/mod.rs`)

### `TypingEngine` Trait
```rust
pub trait TypingEngine: Send + Sync {
    /// Xử lý phím gõ thô và trả về danh sách các ImeAction cần thực hiện
    fn process_key(&mut self, key: &str) -> Vec<ImeAction>;
    
    /// Xóa bộ đệm con trỏ ảo khi di chuyển con trỏ hoặc bấm Space/Enter/Esc
    fn reset(&mut self);
}
```

### `EnglishTestEngine` (Engine gõ Tiếng Anh kiểm thử Sequencer - Decision 023)
- Bẫy phím lặp (ví dụ `aa` -> `A`) để nhả ra cặp lệnh `DeleteSurroundingText(1)` + `CommitString("A")`.
- Kiểm thử và đo đạc phản hồi thực tế của Niri Compositor & Editor Apps qua rào chắn **ACK Barrier**.

### `PassthroughEngine`
- Đóng vai trò bộ gõ passthrough đơn giản.

---

## 3. CHIẾN LƯỢC ĐỊNH HƯỚNG TƯƠNG LAI (LOTUS ENGINE PORT VS FFI)

Theo thảo luận kiến trúc, `src/engine/` ưu tiên phương án **Porting Lotus Engine sang Pure Rust (`LotusRustEngine`)**:
- **Pure Rust 100%:** Zero `unsafe`, zero C-FFI, biên dịch siêu nhẹ bằng `cargo build`.
- **Trải nghiệm gõ cao cấp:** Sở hữu thuật toán khôi phục từ Tiếng Anh (English restore) và kiểm tra chính tả thông minh từ Lotus Engine.


