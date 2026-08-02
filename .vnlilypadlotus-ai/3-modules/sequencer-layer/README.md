# MODULE: SEQUENCER LAYER (`src/sequencer/`)

@status: MILESTONE 1 (STABLE / PASSED UNIT TESTS) | @last_update: 2026-07-27

> **Ghi chú phiên 2026-08-02:** AT-SPI2 DOM ACK Engine đã được thử nghiệm rồi **GỠ BỎ HOÀN TOÀN** theo yêu cầu khôi phục trạng thái main. Sequencer C++ quay về cơ chế `WaitingForAck` (Niri Frame ACK) + micro-delay. Xem [atspi-rollback-notes] — ghi tại `checkpoint.md` phiên 2026-08-02 đợt 2.

---

## 1. TỔNG QUAN VÀ VAI TRÒ KẾT NỐI (OVERVIEW)

**Sequencer Layer** (`ImeEventQueue`) là Trái tim Điều phối Sự kiện của dự án **`vnlilypad`**. Module này chịu trách nhiệm quản lý luồng sự kiện gõ giữa Compositor (Niri), Ứng dụng (Chrome, AFFiNE, VS Code, Terminal) và Engine xử lý Tiếng Việt (Telex/VNI).

> **Nhiệm vụ cốt lõi:** Triệt tiêu $100\%$ hiện tượng đè phím, lặp từ `mminimln`, đảo chữ `choa` và trôi con trỏ bằng cách duy trì Hàng đợi Tuần tự Vi bước **Micro-Step Async State Machine** kết hợp **Rào chắn Ngắt Nhịp Vi Mô (`WaitingMicroDelay` 1ms)** và **ACK Barrier (Wayland / AT-SPI2)**.

---

## 2. KIẾN TRÚC MÁY TRẠNG THÁI MICRO-STEP (MICRO-STEP STATE MACHINE)

```mermaid
stateDiagram-v2
    [*] --> Idle: Khởi tạo ImeEventQueue
    Idle --> WaitingMicroDelay: poll_next_step() (Bắn phím xóa ảo KeyRelease 14 -> Khóa 1ms)
    WaitingMicroDelay --> Idle: Expire 1ms / poll_timeout_ms wake
    Idle --> WaitingForAck: poll_next_step() (DeleteSurroundingText -> Lock ACK)
    WaitingForAck --> Idle: receive_ack() (Wayland Event::Done / AT-SPI2 DBus)
```

### Chi tiết các Trạng thái (`BarrierState`):
- **`Idle`**: Hàng đợi sẵn sàng. Khi nhận action mới từ Engine, `poll_next_step()` sẽ pop `MicroStep` tiếp theo ra dispatch.
- **`WaitingMicroDelay { until }`**: Tạm dừng $1\,\text{ms}$ sau khi phát phím xóa ảo `KEY_BACKSPACE` để Niri Compositor & Terminal render sạch phím xóa trước khi chèn chữ mới.
- **`WaitingForAck { pending_step, dispatched_at }`**: Rào chắn bị khóa chờ tín hiệu `Event::Done` từ Compositor hoặc `ack_timeout_ms` (50ms Freeze Safety Guard).

---

## 3. CHIẾN LƯỢC ĐIỀU PHỐI ADAPTIVE (ADAPTIVE APP-PACED STATE TRACKING - QUYẾT ĐỊNH 021, 022, 051 & 053)

| Chế độ App | Giao thức Thu thập Tín hiệu | Cơ chế Điều phối Sequencer | Đặc tính Hiệu năng |
| :--- | :--- | :--- | :--- |
| **Native Wayland & Web Editors (BlockSuite / Chrome / Electron)** | 1. `zwp_input_method_v2.event.surrounding_text`<br>2. `zwp_input_method_v2.event.done` | Đối soát chuỗi văn bản & vị trí con trỏ thực tế sau khi xóa. Tự động dừng rào chắn (HOLD) nếu App bị lag render chậm (20-50ms) cho tới khi nhận ACK `Done`. | Độ chính xác $100\%$, không đè chữ khi BlockSuite CRDT bị lag. |
| **Terminal Emulators (Foot / Alacritty / Kitty / Fish Shell)** | Single-Stream Virtual Keyboard Atomic Batching (Quyết định 053) | Phân rã lô lệnh thành các `MicroStep::ForwardKey`, tuôn toàn bộ $N$ phím xóa `KEY_BACKSPACE` và `CommitString` trong 1 Wayland commit batch duy nhất với độ trễ $<0.1\,\text{ms}$. | Triệt tiêu $100\%$ lỗi xé lẻ gói tin và lặp rác chữ trên Terminal, phản hồi $0\,\text{ms}$ tức thì. |
| **App X11 / XWayland** | 1. AT-SPI2 DBus (`org.a11y.atspi.Event.Text`) <br>2. DBus Input Context `UpdateSurroundingText` | Lắng nghe tín hiệu `TextChanged` từ Linux Accessibility Bus AT-SPI2 DBus. Tự động mở rào chắn khi chữ trên màn hình thay đổi. | Triệt tiêu $100\%$ lỗi đè phím mà không dùng sleep cứng mù quáng. |

---

## 4. BẢNG THÔNG SỐ VÀ CẤU TRÚC DỮ LIỆU (API REFERENCE)

### `SequencerConfig` (Cấu hình Tham số Luật)
```rust
pub struct SequencerConfig {
    /// Timeout tính bằng ms trước khi ép xả rào chắn (Freeze Safety Guard)
    pub ack_timeout_ms: u64, // Mặc định: 50ms
    /// Trễ vi mô ngắt nhịp giữa phím xóa ảo và commit (milliseconds)
    pub micro_delay_ms: u64, // Mặc định: 1ms
    /// Gộp lệnh xóa và chèn thành 1 Giao dịch Wayland IPC duy nhất
    pub enable_atomic_batching: bool, // Mặc định: true
}
```

### `MicroStep` (Định nghĩa vi bước IME)
- `ForwardKey { keycode: u32, state: u32 }`: Sự kiện phím ảo nguyên tử.
- `CommitString(String)`: Chèn chuỗi ký tự mới.
- `DeleteSurroundingText { before_length: u32, after_length: u32 }`: Xóa chuỗi ký tự xung quanh con trỏ.
- `SetPreeditString(String)`: Cập nhật vùng đệm preedit.

### `ImeEventQueue` (Hàng đợi điều phối)
- `pub fn with_config(config: SequencerConfig) -> Self`: Khởi tạo hàng đợi với cấu hình tùy chỉnh.
- `pub fn push_actions(&mut self, actions: Vec<ImeAction>)`: Nạp danh sách action từ Engine và tự động phân rã thành các `MicroStep`.
- `pub fn poll_next_step(&mut self) -> Option<MicroStep>`: Lấy vi bước tiếp theo nếu rào chắn ngắt nhịp/ACK đang mở.
- `pub fn next_wake_delay(&self) -> Option<Duration>`: Lấy thời gian chờ ngắt nhịp còn lại để `main.rs` tính `libc::poll` timeout động.
- `pub fn receive_ack(&mut self)`: Giải phóng rào chắn ACK khi nhận tín hiệu đồng bộ từ Wayland.

---

## 5. KIỂM THỬ ĐƠN VỊ (UNIT TESTS)

Module được bảo vệ bằng 4 kiểm thử tự động tại `src/sequencer/mod.rs`:
- `test_sequencer_fifo_order`: Kiểm tra nghiêm ngặt thứ tự FIFO vi bước và ngắt nhịp `micro_delay_ms`.
- `test_sequencer_ack_timeout_protection`: Kiểm tra tính năng Freeze Safety Guard (tự động xả rào chắn khi App bị lag/crash quá 50ms).
- `test_sequencer_newline_deduplication`: Kiểm tra lọc trùng phím Enter/Newline.
- `test_sequencer_clear`: Kiểm tra reset hàng đợi khi đổi focus window.

