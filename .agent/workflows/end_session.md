---
description: Kết thúc phiên làm việc fcitx5-lilypad (Niêm phong bộ nhớ & Ghi log)
---

### QUY TRÌNH NGUYÊN TẮC BẮT BUỘC:

Khi người dùng gõ lệnh `/end_session` hoặc yêu cầu đóng phiên làm việc, AI **BẮT BUỘC** phải thực hiện chính xác các bước sau theo thứ tự:

1. **Cập nhật Tài liệu Module (Module Tech Doc Update):**
   - Mở `.fcitx5-lilypad-ai/3-modules/[Tên Module]/README.md` và cập nhật tài liệu kỹ thuật.

2. **Niêm phong Bộ nhớ AI (Memory Checkpoint Update):**
   - Cập nhật `.fcitx5-lilypad-ai/2-memory/decision-log.md` với các quyết định kỹ thuật mới.
   - Cập nhật `.fcitx5-lilypad-ai/1-overview/system_map.md` (chuyển trạng thái module, thêm recent log).
   - Cập nhật `.fcitx5-lilypad-ai/1-overview/project-managers/` lưu tiến độ phase đang làm.
   - Cập nhật `.fcitx5-lilypad-ai/2-memory/checkpoint.md` lưu file dở dang, bug hiện tại, và nhánh Git.

3. **Ghi nhận Tự cải thiện (Self-Improvement Review):**
   - Dự thảo lỗi hành vi phát sinh (nếu có) và xin phép User cập nhật vào `.fcitx5-lilypad-ai/2-memory/self-improve.md`.

4. **Báo cáo hoàn tất (Handover Summary):**
   - Phản hồi: "Sổ bộ nhớ fcitx5-lilypad đã được niêm phong an toàn. Nhánh Git: [Tên Nhánh]. Checkpoint đã lưu. Hệ thống sẵn sàng cho phiên kế tiếp."
