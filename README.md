# 🚀 vnlilypad-lotus (Lotus Upgrade Architecture)

Bộ gõ Tiếng Việt thế hệ mới trên Wayland Compositor (Niri / Sway / Hyprland) dựa trên mô hình kiến trúc **"Nâng cấp Fcitx5 Lotus"**:
**Wayland IPC Commit + Uinput Backspace + Sequencer Layer Orchestration**.

---

## 💡 Điểm Nổi Bật Của Kiến Trúc Mới:

1. **0% EVIOCGRAB (100% Phím Tắt & Phím Chức Năng Hoạt Động Tự Do):**
   - Bộ gõ không cướp quyền phím vật lý ở tầng Kernel (`0% EVIOCGRAB`).
   - Mọi phím tắt (`Ctrl+C`, `Ctrl+V`, `Ctrl+Z`, `Alt+Tab`, `Super+Space`) và phím chức năng (`F1..F12`, Mũi tên, Volume, Brightness) chảy tự do $100\%$ không bao giờ bị đơ hay kẹt.

2. **Giao Thức Wayland IPC Native Commit (Chuẩn Unicode 100%):**
   - Chèn chuỗi Tiếng Việt qua `im.commit_string()` và thanh gạch chân tạm thời qua `im.set_preedit_string()`.
   - Hiển thị văn bản chuẩn Unicode $100\%$, không cần giả lập mã hex hay Alt-code phức tạp.

3. **Uinput Backspace Engine:**
   - `/dev/uinput` chỉ phục vụ mục đích phát $N$ phím xóa `KEY_BACKSPACE` siêu tốc.

4. **Sequencer Layer Orchestration (ACK Barrier Strictness):**
   - Sequencer Layer làm bộ não điều phối vi mô, tạo khoảng dừng $1\,\text{ms}$ giữa lệnh xóa `/dev/uinput` và lệnh chèn `commit_string` Wayland IPC, triệt hạ $100\%$ hiện tượng lặp rác chữ (`mminimln`, `choa`) và đè chữ trên Terminal / Chrome / Electron.

---

## 🛠️ Hướng Dẫn Cài Đặt & Chạy Thử:

### 1. Biên dịch dự án:
```bash
cd /home/chiconcota/Documents/vnlilypad-lotus
cargo build --release
```

### 2. Chạy bộ gõ:
```bash
WAYLAND_DISPLAY=$WAYLAND_DISPLAY XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR ./target/release/vnlilypad
```

### 3. Chạy chế độ Tracer / Dev Mode:
```bash
WAYLAND_DISPLAY=$WAYLAND_DISPLAY XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR ./target/release/vnlilypad --dev
```
