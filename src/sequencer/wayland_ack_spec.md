# WAYLAND MOTION / FRAME ACK ENGINE SPECIFICATION

## 1. Overview
The **Wayland Motion / Frame ACK Engine** works in tandem with the **Sequencer Layer** to eliminate key repeating, stolen frame race conditions, and duplicated text (`chaáo` / `mminimln`) on Linux desktop compositors (such as Niri / Sway / GNOME Wayland).

## 2. Core Mechanisms

### 1. Serial ID Order Enforcement (`serial_counter_`)
- Every transaction (Backspace emission, text commit, modifier update) is assigned a monotonically increasing 32-bit `Serial ID`.
- Stale events from older key presses are discarded immediately (`should_swallow_backspace(serial)`).

### 2. State-Aware ACK Barrier (`BarrierState::WaitingForAck` / `WaitingForDomAck`)
- After emitting raw deletion keys, the Sequencer places an ACK barrier (`WaitingForAck`).
- The text insertion (`CommitString`) is dispatched **only** after receiving the compositor frame ACK (`zwp_input_method_v2.event.done` / `AT-SPI2 TextChanged:delete`).
- Includes a 5ms ~ 15ms dynamic safety timeout to unblock automatically if an application hangs or drops frame notifications.

### 3. Atomic Single Commit
- All micro-steps of a single key transaction are bound together and committed atomically to avoid caret jumping or double commits.
