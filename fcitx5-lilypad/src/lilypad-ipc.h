/*
 * SPDX-FileCopyrightText: 2025-2026 Lý Tất Thành
 * SPDX-FileCopyrightText: 2026 Antigravity
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

#ifndef _FCITX5_LILYPAD_IPC_H_
#define _FCITX5_LILYPAD_IPC_H_

#include <cstdint>

#pragma pack(push, 1)

/**
 * @brief IPC message types between Fcitx5 addon and fcitx5-lilypad-server daemon.
 */
enum LilypadIpcType : uint8_t {
    // Client (Fcitx5) -> Server
    IPC_CMD_BACKSPACE   = 0x01, ///< Delete N characters via Backspace and arm tap
    IPC_CMD_ARM_TAP     = 0x10, ///< Explicit arm hardware tap during deletion
    IPC_CMD_DISARM_TAP  = 0x11, ///< Disarm hardware tap after settling completes
    
    // Server -> Client (Fcitx5)
    IPC_EVT_RESCUED_KEY = 0x20  ///< Rescued physical key from libinput tap
};

/**
 * @brief Atomic datagram packet exchanged over Unix Domain Socket (SOCK_SEQPACKET).
 */
struct LilypadIpcPacket {
    uint8_t  type;          ///< LilypadIpcType
    uint32_t code;          ///< Backspace count OR Linux EVDEV keycode
    uint32_t state;         ///< 1: Pressed, 0: Released (for key events)
    uint64_t timestamp_us;  ///< CLOCK_MONOTONIC timestamp in microseconds
};

#pragma pack(pop)

#endif // _FCITX5_LILYPAD_IPC_H_

