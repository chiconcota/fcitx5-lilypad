---
description: Quy trình sửa lỗi kỹ thuật vknetd (Debugging & Root-cause tracing)
---

1. **Đọc Log Thực tế:**
   - Thu thập log trực tiếp từ stdout/stderr của `vknetd` hoặc từ `journalctl -u vknetd`.
   - Tuyệt đối KHÔNG đưa ra giả thuyết sửa lỗi khi chưa xem un-truncated error log.

2. **Cô lập Nguyên nhân Root Cause:**
   - Phân loại lỗi thuộc:
     - **Kernel Layer:** Lỗi cấp quyền `uinput`, lỗi `EVIOCGRAB` lock, lỗi `SYN_REPORT` timing.
     - **Engine Layer:** Lỗi Telex state machine, lỗi lặp từ, lỗi lệch buffer.
     - **IPC / UI Layer:** Lỗi kết nối Unix domain socket, lỗi tray icon response.

3. **Sửa lỗi & Kiểm chứng:**
   - Thực hiện chỉnh sửa tối thiểu, không gây tác dụng phụ sang module khác.
   - Thử nghiệm runtime hoặc chạy unit test để đảm bảo fix thành công.

4. **Cập nhật Self-Improve:**
   - Ghi lại bài học kinh nghiệm vào `.vknetd-ai/2-memory/self-improve.md`.
