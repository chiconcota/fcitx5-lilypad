/*
 * SPDX-FileCopyrightText: 2025-2026 Lý Tất Thành
 * SPDX-FileCopyrightText: 2026 Antigravity
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
#include <chrono>
#include <sys/un.h>
#include <fcitx-utils/log.h>
#include <fcitx/inputcontext.h>

#include "lilypad-config.h"
#include "lilypad-ipc.h"

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

/**
 * @brief Keystroke record for deduplication buffer.
 */
struct RescuedKeyRecord {
    uint32_t keycode = 0;
    KeySym   sym = 0;
    std::chrono::steady_clock::time_point timestamp{};
};

/**
 * @brief Ring buffer for bi-directional keystroke deduplication.
 */
class DedupRingBuffer {
  private:
    static constexpr size_t CAPACITY = 16;
    RescuedKeyRecord records_[CAPACITY]{};
    size_t head_ = 0;

  public:
    void clear() {
        head_ = 0;
        for (auto& r : records_) {
            r.keycode = 0;
            r.sym = 0;
            r.timestamp = std::chrono::steady_clock::time_point{};
        }
    }

    void record(uint32_t keycode, KeySym sym) {
        records_[head_] = {keycode, sym, std::chrono::steady_clock::now()};
        head_ = (head_ + 1) % CAPACITY;
    }

    bool is_duplicate(uint32_t keycode, KeySym sym, int64_t window_ms = 180) const {
        if (keycode == 0 && sym == 0)
            return false;
        auto now = std::chrono::steady_clock::now();
        for (const auto& rec : records_) {
            bool match = (keycode != 0 && rec.keycode == keycode) ||
                         (sym != 0 && rec.sym == sym);
            if (match) {
                auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - rec.timestamp).count();
                if (elapsed >= 0 && elapsed <= window_ms) {
                    return true;
                }
            }
        }
        return false;
    }

    bool check_and_consume(uint32_t keycode, KeySym sym, int64_t window_ms = 180) {
        if (keycode == 0 && sym == 0)
            return false;
        auto now = std::chrono::steady_clock::now();
        for (auto& rec : records_) {
            bool match = (keycode != 0 && rec.keycode == keycode) ||
                         (sym != 0 && rec.sym == sym);
            if (match) {
                auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - rec.timestamp).count();
                if (elapsed >= 0 && elapsed <= window_ms) {
                    rec.keycode = 0;
                    rec.sym = 0;
                    rec.timestamp = std::chrono::steady_clock::time_point{};
                    return true;
                }
            }
        }
        return false;
    }
};

namespace fcitx {
    class LilypadEngine;
}

// Global state variables for input processing
extern std::atomic<fcitx::LilypadMode> realMode;          ///< Current active input mode
extern std::atomic<bool>             needEngineReset;   ///< Flag to trigger engine reset
extern std::atomic<bool>             g_mouse_clicked;   ///< Mouse click detection flag
extern std::atomic<bool>             is_deleting_;      ///< Deletion in progress flag
extern std::atomic<void*>            g_active_deleting_state; ///< Active LilypadState handling uinput deletion
extern std::atomic<void*>            g_last_deleting_state;   ///< Last active LilypadState for grace period
extern std::atomic<int64_t>          g_last_deleting_end_time_ms; ///< Monotonic ms when is_deleting_ turned false
extern std::atomic<bool>             stop_flag_monitor; ///< Signal to stop monitor threads
extern std::atomic<int>              uinput_client_fd_; ///< Uinput client file descriptor
extern std::atomic<unsigned int>     realtextLen;       ///< Current text length
extern std::atomic<int>              mouse_socket_fd;   ///< Mouse socket file descriptor
extern std::atomic<fcitx::LilypadEngine*> g_lilypad_engine; ///< Global pointer to active engine instance

/**
 * @brief Checks if a given mode is an uinput-based mode.
 * @param mode Mode to check.
 * @return True if mode uses uinput.
 */
inline bool isUinputMode(fcitx::LilypadMode mode) {
    return mode == fcitx::LilypadMode::Sequence ||
           mode == fcitx::LilypadMode::Minecraft;
}

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
 * @brief Converts Linux evdev keycode to XKB/Fcitx KeySym.
 * @param evdev_code Linux evdev keycode.
 * @return Mapped KeySym or FcitxKey_None.
 */
KeySym evdevToKeySym(uint32_t evdev_code);

/**
 * @brief Key event entry for replay buffer.
 */
struct KeyEntry {
    uint32_t sym = 0;   ///< Key symbol
    uint32_t state = 0; ///< Key state (modifiers)
    std::chrono::steady_clock::time_point enter_time = std::chrono::steady_clock::now();
};

/**
 * @brief Queries compositor for focused window app ID (e.g. Niri / Wayland IPC).
 * @return App ID or empty string if not available.
 */
std::string queryCompositorAppId();

#endif // _FCITX5_LILYPAD_UTILS_H_
