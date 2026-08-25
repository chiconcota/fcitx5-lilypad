# 📝 Changelog

Tất cả các thay đổi đáng chú ý của dự án **`fcitx5-lilypad`** được ghi nhận chi tiết tại đây theo chuẩn [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) và [Semantic Versioning](https://semver.org/).

---

## [2.3.0] - 2026-08-26

### 🚀 Tính Năng Mới & Đột Phá Kiến Trúc (New Features & Architecture)

- **Cảm Biến Nhịp Tay Người Dùng (`IIkiSensor` & `StandardIkiSensor`):**
  - Đo liên tục khoảng cách thời gian giữa 2 lần nhấn phím vật lý liên tiếp ($\Delta t = T_n - T_{n-1}$).
  - Áp dụng thuật toán làm mịn EMA ($\text{EMA}_{\text{IKI}} = 0.35 \times \Delta t + 0.65 \times \text{prev}$) để xác định chính xác tốc độ gõ thực tế và nhận diện chế độ gõ siêu tốc (Burst Typing $\le 35\text{ms}$).
- **Dynamic Micro-Pacing via Normalized Lerp & App ACK Consumption:**
  - Áp dụng mô hình chuẩn hóa Min-Max và nội suy tuyến tính (Lerp) kết hợp thời gian tiêu thụ phím xóa của ứng dụng ($N \times T_{\text{ack}}$).
  - Tự động nén vi trễ xuống mức sàn vật lý **$1.5\text{ms} \sim 2.5\text{ms}$** trên Terminal/Alacritty (chuẩn Zero-Latency siêu nhạy).
  - Tự động giãn nở an toàn lên **$45\text{ms} \sim 60\text{ms}$** trên các trình soạn thảo web nặng (Facebook, Google Docs, Discord Web), bảo đảm React DOM kịp tiêu thụ phím xóa trước khi commit ký tự mới.
- **Cold Start Safe Baseline ($>50\text{ms}$):**
  - Thiết lập ngưỡng trần an toàn $50\text{ms} \sim 80\text{ms}$ cho từ đầu tiên khi chưa có lịch sử IKI, loại bỏ $100\%$ nguy cơ nuốt chữ ở ký tự đầu trên các ứng dụng Web DOM nặng.
- **Giao Thức Uinput Sentinel Barrier $N+1$:**
  - Bắn $N+1$ phím xóa `KEY_BACKSPACE` qua `/dev/uinput`: $N$ phím đầu xóa ký tự cũ trên ứng dụng, phím thứ $N+1$ được Fcitx5 nuốt trọn (`filterAndAccept`) làm phím rào chắn an toàn.
  - Bảo đảm trật tự phần cứng FIFO, triệt tiêu $100\%$ race condition (không bao giờ xảy ra tình trạng phím xóa đến sau xóa mất chữ vừa commit).
- **Cơ Chế Two-Tier Timeout & Emergency State Protection:**
  - **Dynamic Soft Timeout ($T_{\text{soft}}$):** Chuyển sang trạng thái `AppLagHolding` và gom phím an toàn vào RAM `buffered_keys_` khi App bị giật/lag DOM.
  - **Watchdog Hard Timeout (250ms) & Emergency Purge:** Main Event Loop cài đặt timer 250ms độc lập. Nếu App bị treo cứng quá 250ms, hệ thống tự động kích hoạt `purgeContextEmergency()` xả toàn bộ phím đệm ra màn hình dưới dạng phím thô (`forwardKey()`), đảm bảo bàn phím **không bao giờ bị đơ/treo**.
- **Uniform Web IME Routing & GTK4 Native Precision ($1\mu\text{s}$):**
  - Tự động đồng bộ luồng commit cho Chromium/Electron chống xung đột Virtual DOM.
  - Giữ nguyên kênh phím Native với độ chính xác timer $1\mu\text{s}$ cho GTK4 / Text Editor, triệt tiêu lỗi đảo dấu cách (`"c òngi"`).

### 📦 Đóng Gói & Triển Khai (Packaging & Deployment)
- Cập nhật phiên bản đồng bộ `2.3.0 (Stable)` trong `CMakeLists.txt`, `about.py` và trang cấu hình GUI.
- Hoàn thiện cấu hình PKGBUILD cho cả 3 gói AUR: `fcitx5-lilypad` (Source), `fcitx5-lilypad-bin` (Binary precompiled), và `fcitx5-lilypad-git` (Git master).
- Viết lại toàn bộ tài liệu giới thiệu đa ngôn ngữ (`README.md`, `fcitx5-lilypad/README.md`, `fcitx5-lilypad/README.en.md`).

---

## [2.2.0] - 2026-08-09

### 🚀 Tính Năng & Cải Tiến (Features & Improvements)
- **Kiến Trúc Modular IAckSensor:** Tách rời lớp cảm biến thành `IAckSensor`, `NiriAckSensor`, `GenericAckSensor`, `AckSensorFactory` tự động nạp theo môi trường `$XDG_CURRENT_DESKTOP`.
- **Nhúng Trực Tiếp 100% Bamboo-Core Go Source:** Tích hợp 19 file mã nguồn Go trực tiếp vào cây thư mục Git chính, loại bỏ hoàn toàn lỗi submodule rỗng khi `git clone`.
- **Tối Ưu Nhịp Replay 3000:300:** Đặt vi trễ $0.3\text{ms}$ cho phím chữ và $3\text{ms}$ cho phím Space, bẻ gãy bẫy đệ quy và loại bỏ hiện tượng đè rác chữ khi gõ siêu tốc.
- **Cờ Release Zero-Log Overhead:** Macro `LILYPAD_INFO` và `LILYPAD_DEBUG` tự động triệt tiêu về `((void)0)` khi biên dịch Release (`NDEBUG`).
- **Bộ Nhận Diện Thương Hiệu Lá Súng Xanh:** Cập nhật bộ icon SVG vector Lá Súng tươi mới (`fcitx-lilypad.svg`) và trang About Qt GUI.

---

## [2.1.0] - 2026-08-04

### 🚀 Tính Năng Cốt Lõi (Core Features)
- **Fcitx5 C++ Sequencer Integration:** Tích hợp máy trạng thái vi bước `MicroStep`, Serial ID Tagging và đếm token xóa nguyên tử.
- **Kernel Uinput Server Daemon (`fcitx5-lilypad-server`):** Daemon độc lập phát phím xóa nguyên tử qua `/dev/uinput` với xác thực UID Unix Socket (`SO_PEERCRED`).
- **Systemd Multi-User Service:** Hỗ trợ kích hoạt per-user qua template service `fcitx5-lilypad-server@.service`.
- **Chế Độ Gõ Sequence (Enum ID 9):** Bổ sung chế độ gõ chuyên biệt tích hợp Sequencer Layer.
