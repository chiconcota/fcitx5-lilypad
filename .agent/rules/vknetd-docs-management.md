---
trigger: always_on
---

# VKNETD DOCUMENTATION MANAGEMENT RULE (NO-MESS POLICY)
@target: All AI Agents | @trigger: /end_session or any documentation update

## 1. THIẾT QUÂN LUẬT KHÔNG RÁC (ZERO-TRASH DIRECTIVE)
- Mọi tài liệu bắt buộc phải nằm gọn bên trong 4 ngăn kéo của `.vknetd-ai/`:

```text
.vknetd-ai/
├── 1-overview/
│   ├── system_map.md           (Bản đồ kiến trúc tổng thể, Trạng thái Module, Change Log)
│   └── project-managers/       (Roadmap & Kế hoạch chi tiết theo các Phase)
├── 2-memory/
│   ├── decision-log.md         (Quy định Kiến trúc lớn, Lịch sử thay đổi cốt lõi)
│   ├── checkpoint.md           (Bàn giao tiến độ dở dang và nhánh Git cho phiên sau)
│   ├── self-improve.md         (Danh sách lỗi hành vi AI cần tránh & Quy tắc tự sửa lỗi)
│   └── archive/                (Lưu trữ nhật ký cũ)
├── 3-modules/                  (Tài liệu kỹ thuật chi tiết theo từng module)
│   ├── kernel-layer/
│   ├── engine-layer/
│   ├── system-layer/
│   └── ui-layer/
└── 4-rules/                    (Quy chuẩn phụ)
```

## 2. QUY TRÌNH GHI ĐÈ CHỐNG LỖI (ANTI-DUPLICATION PROTOCOL)
- Trước khi cập nhật bất kỳ tài liệu nào (`system_map.md`, `decision-log.md`...), AI BẮT BUỘC phải dùng `view_file` hoặc `list_dir` để kiểm tra nội dung hiện tại và sửa trực tiếp (Replace content).
- Nghiêm cấm tự ý tạo file `.md` mới ở Root dự án hoặc thư mục gốc của `.vknetd-ai`.

## 3. LUẬT NGUYÊN TẮC /END_SESSION BẮT BUỘC
- Khi gọi luồng `/end_session`, AI phải cập nhật tối đa 5 điểm chạm:
  1. `1-overview/system_map.md` (Sửa trạng thái & Recent Logs).
  2. `2-memory/decision-log.md` (Ghi quyết định kỹ thuật mới).
  3. `2-memory/checkpoint.md` (Bàn giao tiến độ, file dở dang, lỗi hiện tại, nhánh Git).
  4. `3-modules/[module]/README.md` (Cập nhật tài liệu kỹ thuật module tương ứng).
  5. `1-overview/project-managers/` (Cập nhật tiến độ roadmap).
