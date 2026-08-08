# MODULE: UI LAYER (NATIVE FCITX5 LOTUS SHARED UI)

@status: STABLE (v2.2.0-modular-sensor) | @last_update: 2026-08-08

---

## 1. TỔNG QUAN VÀ SỨ MỆNH (MODULE OVERVIEW)

**UI Layer** của **`fcitx5-lilypad`** dùng chung $100\%$ chuẩn giao diện người dùng gốc từ bộ gõ **Fcitx5 Lotus** (`fcitx5-configtool` & Fcitx5 Core Framework), đảm bảo sự quen thuộc và đồng bộ tuyệt đối cho người dùng Linux.

> **Triết lý thiết kế:** Tận dụng trực tiếp hạ tầng UI chuẩn của Fcitx5 Framework. Bộ gõ không dựng ứng dụng GUI độc lập bên ngoài, toàn bộ cấu hình và menu điều khiển đều tích hợp nguyên tử vào Fcitx5 System Tray và Fcitx5 Configuration Window (`fcitx5-config-qt` / `kcm_fcitx5`).

---

## 2. KIẾN TRÚC VÀ THÀNH PHẦN (COMPONENTS)

### 1. Fcitx5 System Tray Menu (`fcitx5-lilypad/src/lilypad-engine.cpp`)
- **Biểu tượng trạng thái (SubMode Icon):** Hiển thị icon `VIE` (Tiếng Việt) / `ENG` (Tiếng Anh) trực tiếp trên System Tray của Desktop Environment (Niri, KDE, GNOME, Sway, Hyprland, XFCE).
- **Menu Ngữ cảnh (Quick Actions):**
  - Đổi Chế độ gõ (`Sequence`, `Smooth`...).
  - Đổi Bảng mã (`Unicode`, `TCVN3`, `VNI`...).
  - Bật/tắt Kiểm tra chính tả, Macro, Khôi phục từ Tiếng Anh, Từ điển.
- **Menu Quy tắc Ứng dụng (`showAppModeMenu`):** Bật menu gán chế độ gõ cho cửa sổ ứng dụng hiện tại, lưu tự động vào `~/.config/fcitx5/lilypad/app_rules.conf`.

### 2. Native Fcitx5 Configuration UI (`fcitx5-configtool`)
- Giao diện cấu hình chính của `fcitx5-lilypad` xuất hiện trực tiếp trong **Fcitx5 Input Method Configuration**.
- Cho phép người dùng chỉnh sửa phím tắt, tùy chọn nâng cao và cài đặt chung như mọi bộ gõ Fcitx5 tiêu chuẩn.

