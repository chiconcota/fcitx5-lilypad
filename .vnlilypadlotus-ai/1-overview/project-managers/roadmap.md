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

### 🟡 Kế Hoạch Đã & Đang Thực Hiện (Phase 3 — Rebrand, Dual Mode & Sequence Layer UI):

**ĐÃ HOÀN THÀNH (2026-08-02):**
- [x] Đổi tên thương hiệu độc lập **Lotus -> Lilypad** (`liblilypad.so`, `fcitx5-lilypad`, `fcitx5-lilypad-server`).
- [x] Phục hồi 100% mã nguồn `fcitx5-lotus-main/` từ zip archive làm bản tham chiếu gốc, giữ nguyên không sửa.
- [x] Đăng ký song song cả **Lotus** và **Lilypad** trên hệ thống và Fcitx5 active profile/DBus.
- [x] Thêm chế độ gõ **`Sequence`** (UI dropdown, C++ enum `#9`, Python Settings GUI) cho `fcitx5-lilypad`.
- [x] Khởi tạo Git repository mới tinh trên `main`, kết nối remote `git@github.com:chiconcota/fcitx5-lilypad.git` và push 100% code lên GitHub.

**TIẾP THEO (Kế hoạch mở rộng):**
- [ ] Phát triển mã nguồn thực thi thuật toán cho chế độ **`Sequence`** trên `fcitx5-lilypad`.
- [ ] Tối ưu hóa nhịp gõ cho các ứng dụng web/electron nặng (**AFFiNE**, **Facebook Web**).

> **Ghi chú:** Nếu tái khởi động AT-SPI2 sau này, tham khảo commit `aea7094` (docs QD 012) + phiên memory 2026-08-02 đợt 1.
