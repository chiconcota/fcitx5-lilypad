# 🤖 FCITX5 LILYPAD AI AGENT KIT (`.agent/`)

> **Bộ Hướng Dẫn & Quy Trình Tự Động Hóa Cho AI Agent (AI-Assisted Engineering Kit)**

Chào mừng các nhà phát triển và cộng tác viên (Contributors)! Thư mục `.agent/` chứa toàn bộ **Bộ Quy Tắc Kiến Trúc (Rules)** và **Luồng Công Việc (Workflows)** dành riêng cho các AI Coding Assistants (như Gemini Antigravity, Claude Code, Cursor, Windsurf, GitHub Copilot...) khi tham gia phát triển bộ gõ **Fcitx5 Lilypad**.

---

## 📐 1. Cấu Trúc Bộ AI Kit (`.agent/`)

```text
.agent/
├── rules/                                    # Quy chuẩn kiến trúc & quản lý tài liệu
│   ├── fcitx5-lilypad-system-rules.md        # Khẩu quyết tối thượng & quy tắc C++ / uinput
│   └── fcitx5-lilypad-docs-management.md     # Luật Không Rác (Zero-Trash) & Niêm phong bộ nhớ
├── workflows/                                # Lệnh viết tắt (Slash Commands) điều phối AI
│   ├── start_session.md                      # Lệnh /start_session: Nạp bộ nhớ AI khi bắt đầu
│   ├── end_session.md                        # Lệnh /end_session: Niêm phong & lưu checkpoint
│   ├── debug-workflow.md                     # Lệnh /debug-workflow: Quy trình truy vết log bug
│   └── cuu_toi.md                            # Lệnh /cuu_toi: Cứu hộ khẩn cấp khi gặp sự cố
└── README.md                                 # Tài liệu hướng dẫn sử dụng AI Kit (File này)
```

---

## 🚀 2. Các Lệnh Workflow Viết Tắt (Slash Commands)

Khi làm việc với AI Agent trong dự án này, bạn có thể gọi trực tiếp các **Slash Commands** sau để AI tự động kích hoạt luồng công việc chuẩn:

| Command | Mô tả & Tác dụng | Khi nào sử dụng? |
| :--- | :--- | :--- |
| **`/start_session`** | Kích hoạt AI đọc bộ nhớ từ `.fcitx5-lilypad-ai/` (`checkpoint.md`, `system_map.md`, `self-improve.md`), xác định nhánh Git và cô lập Module cần làm việc. | **Bắt đầu** mỗi phiên làm việc mới với AI. |
| **`/end_session`** | Kích hoạt AI tự động cập nhật tài liệu kỹ thuật Module, niêm phong quyết định kiến trúc, ghi checkpoint tiến độ và báo cáo bàn giao. | **Kết thúc** phiên làm việc trước khi push/commit. |
| **`/debug-workflow`** | Yêu cầu AI trích xuất log thực tế (`journalctl` / stdout), phân tích Root Cause trước khi đưa ra phương án sửa lỗi. | Khi gặp bug gõ chữ, kẹt phím hoặc crash. |
| **`/cuu_toi`** | Luồng cứu hộ khẩn cấp khôi phục mã nguồn C++ sạch về trạng thái an toàn. | Khi code bị panic hoặc bàn phím bị treo. |

---

## 🧠 3. Hệ Thống Bộ Nhớ 4 Ngăn Kéo (`.fcitx5-lilypad-ai/`)

Để tránh hiện tượng AI bị "trôi bộ nhớ" hoặc tạo tài liệu rác làm bẩn Repository, dự án quy định **Thiết Quân Luật Không Rác (Zero-Trash Directive)**: Tất cả tài liệu dự án BẮT BUỘC nằm gọn trong 4 ngăn kéo:

1. **`1-overview/`**:
   - `system_map.md`: Bản đồ kiến trúc tổng thể, Bảng trạng thái Module và Lịch sử nâng cấp dòng thời gian.
   - `project-managers/`: Roadmap chi tiết theo từng Phase.
2. **`2-memory/`**:
   - `checkpoint.md`: Bàn giao tiến độ dở dang, file đang sửa, lỗi hiện tại và nhánh Git.
   - `decision-log.md`: Nhật ký lưu trữ các Quyết định Kiến trúc cốt lõi đang vận hành 100%.
   - `self-improve.md`: Danh sách bài học kinh nghiệm tự sửa lỗi hành vi AI cần tránh.
   - `archive/`: Lưu trữ các báo cáo phiên cũ hoặc thử nghiệm đã gỡ bỏ.
3. **`3-modules/`**: Tài liệu đặc tả kỹ thuật chi tiết theo từng Module Layer (`engine-layer`, `kernel-layer`, `sequencer-layer`, `ui-layer`).
4. **`4-rules/`**: Các quy chuẩn phụ của dự án.

---

## ⚡ 4. Nguyên Tắc Cốt Lõi Cho AI Agent (Prime Directives)

Mọi AI Agent làm việc trên mã nguồn **Fcitx5 Lilypad** bắt buộc phải tuân thủ 6 Khẩu Quyết Tối Thượng:

1. **Hybrid Fcitx5 C++ Integration:** Tích hợp 100% qua Fcitx5 C++ Addon API cho Wayland/X11 IPC. Tầng xóa vi mô phát phím Backspace qua daemon `/dev/uinput` riêng.
2. **Panic Safety & No-Freeze Guarantee:** Mọi thao tác gán phím hoặc rào chắn ACK barrier phải có rào chắn trần **250ms** tự động giải phóng chống treo phím.
3. **Word Boundary Strictness:** Tất cả các ký tự đặc biệt, phím điều hướng, Space, Enter, Tab, Esc... BẮT BUỘC lập tức XÓA RỖNG (RESET) Virtual Caret Buffer.
4. **Race Condition Prevention:** Khi phát lại chuỗi phím qua uinput, áp dụng Batch Replay Protocol ($0.1\text{ms}$ phím chữ, $3\text{ms}$ Space) để tránh lặp từ trên các ứng dụng nặng (VS Code, Chrome, Electron).
5. **Decoupled Architecture (IPC Only):** Tiến trình Fcitx5 Addon và Pure Uinput Server Daemon cách ly hoàn toàn qua Unix Domain Socket (`/run/user/$UID/fcitx5-lilypad.sock`) có xác thực `SO_PEERCRED`.
6. **No Code Edits Without Asking on Error Report:** Nếu người dùng báo lỗi, AI BẮT BUỘC phải hỏi ý kiến người dùng và thu thập log chứng cứ trước khi chạm vào mã nguồn.

---

## 🛠️ 5. Cách Đóng Góp Cho AI Kit

Nếu bạn muốn bổ sung Workflow mới hoặc tinh chỉnh Rule cho AI:
1. Thêm file rule mới vào `.agent/rules/` với YAML frontmatter `trigger: always_on`.
2. Thêm file workflow mới vào `.agent/workflows/<command-name>.md`.
3. Cập nhật bảng tổng hợp lệnh vào file `README.md` này.
