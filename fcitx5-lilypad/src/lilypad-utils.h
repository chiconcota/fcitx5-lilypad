/*
 * SPDX-FileCopyrightText: 2025 Võ Ngô Hoàng Thành <thanhpy2009@gmail.com>
 * SPDX-FileCopyrightText: 2026 Nguyễn Hoàng Kỳ  <nhktmdzhg@gmail.com>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/**
 * @file lilypad-utils.h
 * @brief Utility functions and global state for fcitx5-lilypad.
 */

#ifndef _FCITX5_LILYPAD_UTILS_H_
#define _FCITX5_LILYPAD_UTILS_H_

#include <atomic>
#include <sys/un.h>
#include <fcitx-utils/log.h>
#include <fcitx/inputcontext.h>

#include "lilypad-config.h"

/**
 * @brief Maximum length of Unix socket paths.
*/
#define UNIX_PATH_MAX sizeof(((struct sockaddr_un*)0)->sun_path)

FCITX_DECLARE_LOG_CATEGORY(lilypad);

#if defined(NDEBUG) || defined(DISABLE_LILYPAD_LOGS)
#define LILYPAD_DEBUG(msg) ((void)0)
#define LILYPAD_INFO(msg)  ((void)0)
#define LILYPAD_WARN(msg)  FCITX_LOGC(lilypad, Warn) << "[WARN] " << msg
#define LILYPAD_ERROR(msg) FCITX_LOGC(lilypad, Error) << "[ERROR] " << msg
#else
#define LILYPAD_DEBUG(msg) FCITX_LOGC(lilypad, Debug) << "[DEBUG] " << msg
#define LILYPAD_INFO(msg)  FCITX_LOGC(lilypad, Info) << "[INFO] " << msg
#define LILYPAD_WARN(msg)  FCITX_LOGC(lilypad, Warn) << "[WARN] " << msg
#define LILYPAD_ERROR(msg) FCITX_LOGC(lilypad, Error) << "[ERROR] " << msg
#endif

// Forward declaration for fcitx types
using KeySym = uint32_t;

// Global state variables for input processing
extern std::atomic<fcitx::LilypadMode> realMode;          ///< Current active input mode
extern std::atomic<bool>             needEngineReset;   ///< Flag to trigger engine reset
extern std::atomic<bool>             g_mouse_clicked;   ///< Mouse click detection flag
extern std::atomic<bool>             is_deleting_;      ///< Deletion in progress flag
extern std::atomic<bool>             stop_flag_monitor; ///< Signal to stop monitor threads
extern std::atomic<int>              uinput_client_fd_; ///< Uinput client file descriptor
extern std::atomic<unsigned int>     realtextLen;       ///< Current text length
extern std::atomic<int>              mouse_socket_fd;   ///< Mouse socket file descriptor

/**
 * @brief Builds socket path from base suffix.
 * @param base_path_suffix Suffix to append to base path.
 * @return Full socket path.
 */
std::string buildSocketPath(const char* base_path_suffix);

/**
 * @brief Gets current time in milliseconds.
 * @return Timestamp in milliseconds.
 */
int64_t now_ms();

/**
 * @brief Checks if key symbol is a backspace.
 * @param sym Key symbol to check.
 * @return True if backspace.
 */
bool isBackspace(uint32_t sym);

/**
 * @brief Compares two strings and computes diff.
 * @param A First string.
 * @param B Second string.
 * @param deletedPart Output deleted portion.
 * @param addedPart Output added portion.
 * @return Comparison result code.
 */
int compareAndSplitStrings(const std::string& A, const std::string& B, std::string& deletedPart, std::string& addedPart);

/**
 * @brief Checks if string starts with prefix.
 * @param str String to check.
 * @param prefix Prefix to check.
 * @return True if string starts with prefix.
 */
bool isStartsWith(const std::string& str, const std::string& prefix);

/**
 * @brief Get the frontend name from the input context.
 * @param ic Input context.
 * @return Frontend name.
 */
std::string getFrontendName(fcitx::InputContext* ic);

/**
 * @brief Key event entry for replay buffer.
 */
struct KeyEntry {
    uint32_t sym;   ///< Key symbol
    uint32_t state; ///< Key state (modifiers)
};

#endif // _FCITX5_LILYPAD_UTILS_H_
