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
- [ ] **Phase 5: Merge nhánh `feat/iki-adaptive-engine` vào `main` & Phát hành `v2.3.0`**.

