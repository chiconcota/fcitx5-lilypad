---
description: Kết thúc phiên làm việc vknetd (Niêm phong bộ nhớ & Ghi log)
---

1. **Tổng hợp Kiến thức:**
   - Rà soát lại tất cả các file mã nguồn vừa chỉnh sửa hoặc tạo mới.
   - Xác định các thay đổi struct, API, IPC protocol hoặc quy tắc State Machine.

2. **Cập nhật Module Documentation:**
   - Mở `.vknetd-ai/3-modules/[Tên Module]/README.md` và cập nhật tài liệu kỹ thuật.

3. **Niêm phong Bộ nhớ:**
   - Cập nhật `.vknetd-ai/2-memory/decision-log.md` với các quyết định kỹ thuật mới.
   - Cập nhật `.vknetd-ai/1-overview/system_map.md` (chuyển trạng thái module, thêm recent log).
   - Cập nhật `.vknetd-ai/1-overview/project-managers/` lưu tiến độ phase đang làm.
   - Cập nhật `.vknetd-ai/2-memory/checkpoint.md` lưu file dở dang, bug hiện tại, và nhánh Git.
   - Hỏi User ý kiến về việc `git commit` và `push`.

4. **Interactive Review & Self-Improve:**
   - Dự thảo lỗi hành vi phát sinh (nếu có) và xin phép User cập nhật vào `.vknetd-ai/2-memory/self-improve.md`.

5. **Xác nhận Kết thúc:**
   - Phản hồi: "Sổ bộ nhớ vknetd đã được niêm phong an toàn. Nhánh Git: [Tên Nhánh]. Checkpoint đã lưu. Hệ thống sẵn sàng cho phiên kế tiếp."
