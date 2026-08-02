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

### 🟡 Kế Hoạch Phiên Tới (Phase 3 — Heavy & Slow Apps Optimization):

**ĐÃ LÀM (2026-08-02):**
- [x] Đã thử nghiệm AT-SPI2 DOM ACK Engine → **GỠ BỎ HOÀN TOÀN** theo yêu cầu khôi phục main (build pass 100%). Lý do: code ngoài git không revert được, phải gỡ tay.
- [x] Xác lập quy tắc: mọi thay đổi `fcitx5-lotus-main/` phải ghi chi tiết vào checkpoint để có thể tái dựng.

**CHƯA LÀM (kế hoạch gốc Phase 3, chưa triển khai lại):**
- [ ] Tối ưu hóa bộ gõ cho các ứng dụng nhả chữ chậm / lag nặng: **AFFiNE (BlockSuite Editor)**, **Facebook Web / Messenger Web**.
- [ ] Adaptive Dynamic Barrier Delay cho Sequencer C++ khi nhả chữ trên app lag.
- [ ] Unit Tests bổ sung cho Sequencer C++ (`lotus-sequencer.cpp`).
- [ ] Release `v2.1.0-lotus` với Sequencer Token Swallow V2.

> **Ghi chú:** Nếu tái khởi động AT-SPI2 sau này, tham khảo commit `aea7094` (docs QD 012) + phiên memory 2026-08-02 đợt 1.
