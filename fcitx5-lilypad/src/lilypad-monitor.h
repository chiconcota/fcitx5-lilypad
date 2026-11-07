/*
 * SPDX-FileCopyrightText: 2025-2026 Lý Tất Thành
 * SPDX-FileCopyrightText: 2026 Antigravity
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/**
 * @file lilypad-monitor.h
 * @brief Input monitoring and timing utilities for fcitx5-lilypad.
 */

#ifndef _FCITX5_LILYPAD_MONITOR_H_
#define _FCITX5_LILYPAD_MONITOR_H_

#include <thread>

extern std::thread mouse_thread;

/**
 * @brief Thread function for mouse press detection and reset.
 *
 * Monitors mouse events to reset input state when needed.
 */
void mousePressResetThread();

/**
 * @brief Starts the mouse reset monitoring.
 */
void startMouseReset();

#endif // _FCITX5_LILYPAD_MONITOR_H_
