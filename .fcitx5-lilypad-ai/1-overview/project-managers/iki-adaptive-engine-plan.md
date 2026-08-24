# KẾ HOẠCH QUẢN LÝ DỰ ÁN: IKI ADAPTIVE ENGINE & TWO-TIER TIMEOUT (PHASE 4)

> **Nhánh Git phụ trách:** `feat/iki-adaptive-engine`
> **Mục tiêu:** Đưa `fcitx5-lilypad` đạt đẳng cấp **Zero-Latency (Gõ lướt không trễ)** khi máy mượt và **Zero-Corruption (Không sai/hỏng chữ)** khi máy/app bị lag.
> **Kiến trúc:** IKI Passive Measurement + Dynamic Micro-pacing + Two-Tier Timeout (Soft/Hard).

---

## 🧭 TỔNG QUAN 3 GIAI ĐOẠN (3 PHASES BREAKDOWN)

```text
┌────────────────────────────────────────────────────────────────────────┐
│               PHASE 4.1: PASSIVE IKI MEASUREMENT (COMMIT 1)            │
│  - Thu thập nhịp gõ thời gian thực delta_t giữa 2 phím nhấn liên tiếp  │
│  - Thuật toán EMA (Exponential Moving Average) tính IKI trung bình     │
│  - Tạo cờ an toàn enableIkiAdaptive trong lilypad-config.h             │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ (Cung cấp chỉ số IKIavg)
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│             PHASE 4.2: DYNAMIC MICRO-PACING OPTIMIZATION (COMMIT 2)    │
│  - Ép nhịp micro-delay xuống 1ms ~ 2ms khi IKI < 30ms và App mượt      │
│  - Xóa bỏ hoàn toàn "chờ mù" (Blind Wait 14ms~18ms không cần thiết)    │
│  - Tối ưu nhịp Micro-Bursting (1 BS = 1 SYN_REPORT với 1ms pacing)     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ (Bảo vệ khi App Lag: T_roundtrip > IKI)
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│        PHASE 4.3: TWO-TIER TIMEOUT & STATE PROTECTION (COMMIT 3)       │
│  - Soft Timeout (IKI * 2.0 ~ 40ms-80ms): Giữ phím RAM, tạm hoãn uinput │
│  - Hard Timeout (200ms): Khẩn cấp cắt lỗ Context, xả thô bảo vệ text   │
│  - Kiểm thử ma trận toàn diện: Electron, Chrome, Terminal, LibreOffice │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 📌 CHI TIẾT TỪNG PHASE

### 🔹 PHASE 4.1: PASSIVE IKI MEASUREMENT (ĐO TỐC ĐỘ GÕ NGẦM)
- **Files cần sửa:**
  - `fcitx5-lilypad/src/lilypad-config.h`: Thêm cờ `enableIkiAdaptive` (mặc định `true`), `ikiMinMs` (10ms), `ikiMaxMs` (500ms).
  - `fcitx5-lilypad/src/lilypad-state.h`: Khai báo biến `last_physical_key_time_`, `current_iki_ms_`, `iki_ema_ms_`.
  - `fcitx5-lilypad/src/lilypad-state.cpp`:
    - Trong `LilypadState::keyEvent()`, tính $\Delta t = \text{now} - \text{last\_physical\_key\_time\_}$ đối với phím ký tự gõ thật (bỏ qua phím modifier hoặc phím release).
    - Cập nhật EMA: $\text{IKI}_{\text{new}} = 0.3 \times \Delta t + 0.7 \times \text{IKI}_{\text{prev}}$ (nếu $\Delta t \le 1000\text{ms}$).
- **Tiêu chí Hoàn thành (DoD):**
  - Biên dịch không có warning/error.
  - Gõ thử trong Terminal và xem log: Số đo IKI phản ánh chính xác khi gõ chậm ($\approx 150\text{ms}$) và khi gõ nhanh ($\approx 20\text{ms}$).

---

### 🔹 PHASE 4.2: DYNAMIC MICRO-PACING (TỐI ƯU NHỊP DELAY SIÊU TỐC)
- **Files cần sửa:**
  - `fcitx5-lilypad/src/ack-sensors/ack-sensor.h` & `niri-sensor.cpp`:
    - Cho phép hàm `get_micro_delay_us(int bsCount, uint64_t iki_ms)` nhận thêm tham số IKI.
    - Khi $\text{IKI} < 30\text{ms}$ (gõ lướt cực nhanh) và $T_{\text{measured}} \le \text{IKI}$:
      - `micro_delay_us = 1000 + bsCount * 1000` (1ms ~ 3ms thay vì 6ms ~ 18ms cũ).
  - `fcitx5-lilypad/src/lilypad-state.cpp`:
    - Truyền `current_iki_ms_` vào hàm tính `micro_delay_us` trước khi tạo timer commit.
- **Tiêu chí Hoàn thành (DoD):**
  - Cảm giác gõ các từ có dấu phức tạp (`hoàng`, `nghiêng`, `đường`) trên máy mượt có cảm giác tức thì $100\%$ (Zero-Latency feel).
  - Không bị rơi rớt hay nuốt chữ khi gõ tổ hợp phím cực nhanh.

---

### 🔹 PHASE 4.3: TWO-TIER TIMEOUT & CONTEXT INVALIDATION (CHỐNG SAI CHỮ KHI APP LAG)
- **Files cần sửa:**
  - `fcitx5-lilypad/src/lilypad-sequencer.h/.cpp`:
    - Cấu hình 2 mốc timeout: `soft_timeout_ms_` ($2.0 \times \text{IKI}$) và `hard_timeout_ms_` ($200\text{ms}$).
    - Nếu quá `soft_timeout_ms_`: Không xả phím vội, chuyển trạng thái `BarrierState::AppLagHolding` để `LilypadState` tiếp tục giữ chặt phím mới trong `buffered_keys_`.
    - Nếu quá `hard_timeout_ms_` ($200\text{ms}$): Kích hoạt `purge_context_emergency()`, reset word buffer, xả phím trong RAM dạng raw phím thô để người dùng tiếp tục gõ bình thường mà không bị kẹt.
- **Tiêu chí Hoàn thành (DoD):**
  - Test trên các app nặng (VS Code khi indexing, Chrome tải trang nặng, Slack/Discord): Không xuất hiện chữ rác khi app bị lag giật khung hình.
  - Phím không bao giờ bị đơ/freeze quá 200ms.

---

## 📊 TIẾN ĐỘ THỰC HIỆN (EXECUTION CHECKLIST)

| Task ID | Nội dung công việc | Phụ trách | Trạng thái |
| :--- | :--- | :--- | :--- |
| **TASK-401** | Tạo Git Branch `feat/iki-adaptive-engine` & cập nhật Project Manager | AI / User | 🟢 **ĐÃ XONG** |
| **TASK-402** | Phase 4.1: Viết mã nguồn Modular IIkiSensor Layer & Feature Flag | AI / User | 🟢 **ĐÃ XONG** |
| **TASK-403** | Phase 4.1: Build, cài đặt và kiểm thử log chỉ số IKI thực tế | User / AI | 🟢 **ĐÃ XONG (Chính xác 100%)** |
| **TASK-404** | Phase 4.2: Tích hợp Dynamic Micro-Pacing theo nhịp IKI | AI / User | 🟡 Sẵn sàng cho phiên sau |
| **TASK-405** | Phase 4.2: Kiểm thử tốc độ Zero-Latency trên Terminal / Chrome | User | ⚪ Chờ thực hiện |
| **TASK-406** | Phase 4.3: Xây dựng Two-Tier Timeout (Soft/Hard) & Context Invalidation | AI / User | ⚪ Chờ thực hiện |
| **TASK-407** | Phase 4.3: Kiểm thử tải nặng và chống sai chữ khi App lag | User | ⚪ Chờ thực hiện |
| **TASK-408** | Merge nhánh `feat/iki-adaptive-engine` vào `main`, bump `v2.3.0` | AI / User | ⚪ Chờ thực hiện |
