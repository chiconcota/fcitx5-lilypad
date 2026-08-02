---
description: Bắt đầu phiên làm việc chuẩn vknetd (Load Context & Memory)
---

1. **Khởi động bộ nhớ (Context Loading):**
   - Đọc `.vknetd-ai/2-memory/checkpoint.md` ĐẦU TIÊN để nhận bàn giao trạng thái công việc, nhánh Git, và bug dở dang.
   - Đọc `.vknetd-ai/1-overview/system_map.md` để nắm cấu trúc tổng thể và trạng thái từng Module.
   - Đọc `.vknetd-ai/2-memory/decision-log.md` để biết những quyết định thiết kế gần nhất.
   - Đọc `.vknetd-ai/2-memory/self-improve.md` để nạp danh sách lỗi hành vi cần tránh.

2. **Xác định tiêu điểm & Nhánh làm việc:**
   - Kiểm tra nhánh Git hiện tại (`git branch` hoặc `git status`).
   - Xác định Module mục tiêu (Kernel Layer, Engine Layer, System Layer, hay UI Layer).

3. **Load Module Memory (Context Isolation):**
   - Đọc TOÀN BỘ tài liệu trong `.vknetd-ai/3-modules/[Tên Module]/`.
   - Tuyệt đối KHÔNG đọc lộn tài liệu của Module không liên quan để tránh tràn bộ nhớ context.

4. **Sẵn sàng (Ready Check):**
   - Phản hồi ngắn gọn: "Đã nạp kiến trúc vknetd (system_map.md) và Nhật ký tự cải thiện (self-improve.md). Đang làm việc trên nhánh Git: [Tên Nhánh]. Đang cô lập vùng làm việc vào Module: [Tên Module]. Sẵn sàng nhận lệnh."
