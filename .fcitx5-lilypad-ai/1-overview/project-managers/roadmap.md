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

- [ ] **Phase 4.1: Passive IKI Measurement (Đo IKI ngầm 0% Overhead)**
  - [ ] Thêm biến đo $\Delta t = T_n - T_{n-1}$ trong `LilypadState::keyEvent()`.
  - [ ] Tính `current_iki_ms_` qua thuật toán EMA smoothing.
  - [ ] Thêm cờ cấu hình `enableIkiAdaptive` trong `lilypad-config.h`.
- [ ] **Phase 4.2: Dynamic Micro-Pacing Optimization**
  - [ ] Điều chỉnh động `micro_delay_us` theo $\min(IKI, D_{app})$ khi App mượt ($T_{roundtrip} \le IKI$).
  - [ ] Ép nhịp micro-pacing về $1\text{ms} \sim 2\text{ms}$ khi gõ lướt siêu tốc (Zero-Latency feel).
- [ ] **Phase 4.3: Two-Tier Timeout & Context Invalidation (State Protection)**
  - [ ] Thiết lập **Soft Timeout** ($IKI \times 2.0 \approx 40\text{ms} - 80\text{ms}$): Giữ phím trong RAM (`buffered_keys_`), tạm dừng phát uinput mới.
  - [ ] Thiết lập **Hard Timeout** ($150\text{ms} - 200\text{ms}$): Cắt lỗ trạng thái khẩn cấp, purge word buffer và xả phím thô an toàn khi app bị freeze.
  - [ ] Kiểm thử toàn diện trên Web / Electron (Chrome, VS Code, Discord, Slack, Messenger, Terminal).

