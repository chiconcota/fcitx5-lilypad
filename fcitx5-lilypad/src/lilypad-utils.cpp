/*
 * SPDX-FileCopyrightText: 2025-2026 Lý Tất Thành
 * SPDX-FileCopyrightText: 2026 Antigravity
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */
#include "lilypad-utils.h"
#include "lilypad-config.h"

#include <cstddef>
#include <cstring>
#include <sys/socket.h>
#include <fcitx-utils/utf8.h>
#include <pwd.h>
#include <unistd.h>

#include <algorithm>
#include <chrono>
#include <linux/input-event-codes.h>

// Global variables
std::atomic<fcitx::LilypadMode> realMode{fcitx::LilypadMode::Sequence};
std::atomic<bool>             needEngineReset{false};
std::atomic<bool>             g_mouse_clicked{false};
std::atomic<bool>             is_deleting_{false};
std::atomic<void*>            g_active_deleting_state{nullptr};
std::atomic<void*>            g_last_deleting_state{nullptr};
std::atomic<int64_t>          g_last_deleting_end_time_ms{0};
std::atomic<bool>             stop_flag_monitor{false};
std::atomic<int>              uinput_client_fd_{-1};
std::atomic<unsigned int>     realtextLen{0};
std::atomic<int>              mouse_socket_fd{-1};
std::atomic<fcitx::LilypadEngine*> g_lilypad_engine{nullptr};

FCITX_DEFINE_LOG_CATEGORY(lilypad, "lilypad", fcitx::LogLevel::Debug);

std::string buildSocketPath(const char* base_path_suffix) {
    struct passwd  pwd{};
    struct passwd* result   = nullptr;
    long           buf_size = sysconf(_SC_GETPW_R_SIZE_MAX);
    if (buf_size == -1) {
        buf_size = 16384;
    }
    std::vector<char> buf(buf_size);
    std::string       username;
    int               res = getpwuid_r(getuid(), &pwd, buf.data(), buf_size, &result);
    if (res == 0 && result != nullptr) {
        username = result->pw_name;
    } else {
        username = "unknown";
    }
    std::string path;
    path.reserve(32);
    path += "lilypadsocket-";
    path += username;
    path += '-';
    path += base_path_suffix;
    const size_t max_socket_path_length = UNIX_PATH_MAX - 1;
    path.resize(std::min(path.length(), max_socket_path_length));
    return path;
}

int64_t now_ms() {
    return std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now().time_since_epoch()).count();
}

bool isBackspace(uint32_t sym) {
    return sym == 65288 || sym == 8 || sym == FcitxKey_BackSpace;
}

int compareAndSplitStrings(const std::string& A, const std::string& B, std::string& deletedPart, std::string& addedPart) {
    size_t i = 0;
    size_t j = 0;
    while (i < A.size() && j < B.size()) {
        unsigned int lenA = fcitx_utf8_char_len(&A[i]);
        unsigned int lenB = fcitx_utf8_char_len(&B[j]);
        if (lenA == 0 || lenB == 0) {
            break;
        }
        if (i + lenA > A.size() || j + lenB > B.size()) {
            break;
        }
        if (lenA == lenB && std::strncmp(&A[i], &B[j], lenA) == 0) {
            i += lenA;
            j += lenB;
        } else {
            break;
        }
    }

    deletedPart.assign(A, i);
    addedPart.assign(B, j);
    return (deletedPart.empty() && addedPart.empty()) ? 1 : 2;
}

bool isStartsWith(const std::string& str, const std::string& prefix) {
#if __cplusplus >= 202002L
    return str.starts_with(prefix);
#else
    return str.substr(0, prefix.size()) == prefix;
#endif
}

std::string getFrontendName(fcitx::InputContext* ic) {
    if (ic == nullptr) {
        return "unknown";
    }
    return ic->frontend();
}

KeySym evdevToKeySym(uint32_t evdev_code) {
    if (evdev_code >= KEY_1 && evdev_code <= KEY_9) {
        return FcitxKey_1 + (evdev_code - KEY_1);
    }
    if (evdev_code == KEY_0) {
        return FcitxKey_0;
    }

    switch (evdev_code) {
        case KEY_Q: return FcitxKey_q;
        case KEY_W: return FcitxKey_w;
        case KEY_E: return FcitxKey_e;
        case KEY_R: return FcitxKey_r;
        case KEY_T: return FcitxKey_t;
        case KEY_Y: return FcitxKey_y;
        case KEY_U: return FcitxKey_u;
        case KEY_I: return FcitxKey_i;
        case KEY_O: return FcitxKey_o;
        case KEY_P: return FcitxKey_p;
        case KEY_A: return FcitxKey_a;
        case KEY_S: return FcitxKey_s;
        case KEY_D: return FcitxKey_d;
        case KEY_F: return FcitxKey_f;
        case KEY_G: return FcitxKey_g;
        case KEY_H: return FcitxKey_h;
        case KEY_J: return FcitxKey_j;
        case KEY_K: return FcitxKey_k;
        case KEY_L: return FcitxKey_l;
        case KEY_Z: return FcitxKey_z;
        case KEY_X: return FcitxKey_x;
        case KEY_C: return FcitxKey_c;
        case KEY_V: return FcitxKey_v;
        case KEY_B: return FcitxKey_b;
        case KEY_N: return FcitxKey_n;
        case KEY_M: return FcitxKey_m;
        case KEY_SPACE: return FcitxKey_space;
        case KEY_MINUS: return FcitxKey_minus;
        case KEY_EQUAL: return FcitxKey_equal;
        case KEY_LEFTBRACE: return FcitxKey_bracketleft;
        case KEY_RIGHTBRACE: return FcitxKey_bracketright;
        case KEY_SEMICOLON: return FcitxKey_semicolon;
        case KEY_APOSTROPHE: return FcitxKey_apostrophe;
        case KEY_GRAVE: return FcitxKey_grave;
        case KEY_BACKSLASH: return FcitxKey_backslash;
        case KEY_COMMA: return FcitxKey_comma;
        case KEY_DOT: return FcitxKey_period;
        case KEY_SLASH: return FcitxKey_slash;
        case KEY_BACKSPACE: return FcitxKey_BackSpace;
        case KEY_ENTER: return FcitxKey_Return;
        default: break;
    }
    return FcitxKey_None;
}

std::string queryCompositorAppId() {
    const char* sockPath = getenv("NIRI_SOCKET");
    if (!sockPath || sockPath[0] == '\0') {
        return "";
    }
    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) return "";

    struct timeval tv;
    tv.tv_sec = 0;
    tv.tv_usec = 15000; // 15ms safety timeout
    setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
    setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv));

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, sockPath, sizeof(addr.sun_path) - 1);

    if (connect(fd, reinterpret_cast<struct sockaddr*>(&addr), sizeof(addr)) < 0) {
        close(fd);
        return "";
    }

    const char req[] = "\"FocusedWindow\"\n";
    if (write(fd, req, sizeof(req) - 1) < 0) {
        close(fd);
        return "";
    }

    char buf[2048];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) return "";
    buf[n] = '\0';

    std::string resp(buf, n);
    auto pos = resp.find("\"app_id\":");
    if (pos == std::string::npos) return "";
    pos += 9;
    while (pos < resp.size() && (resp[pos] == ' ' || resp[pos] == '"')) pos++;
    auto endPos = resp.find('"', pos);
    if (endPos == std::string::npos) return "";
    return resp.substr(pos, endPos - pos);
}
