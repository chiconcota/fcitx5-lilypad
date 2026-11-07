/*
 * SPDX-FileCopyrightText: 2022-2022 CSSlayer <wengxt@gmail.com>
 * SPDX-FileCopyrightText: 2025-2026 Lý Tất Thành
 * SPDX-FileCopyrightText: 2026 Antigravity
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/**
 * @file lilypad-state-uinput.cpp
 * @brief Uinput socket IPC client, Sentinel key handling, Autofill guard & Focus state.
 */

#include "lilypad-state.h"
#include "lilypad-engine.h"
#include "lilypad-candidates.h"
#include "lilypad-utils.h"
#include "lilypad.h"

#include <cstddef>
#include <fcitx-utils/log.h>
#include <fcitx-utils/utf8.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputpanel.h>
#include <fcitx/userinterface.h>

#include <algorithm>
#include <string>
#include <sys/socket.h>
#include <sys/un.h>
#include <thread>

namespace fcitx {

    static std::unique_ptr<EventSourceIO> g_uinput_io_event;

    bool LilypadState::connect_uinput_server() {
        if (uinput_client_fd_ >= 0)
            return true;
        const std::string current_path = buildSocketPath("kb_socket");
        int               current_fd   = socket(AF_UNIX, SOCK_SEQPACKET | SOCK_NONBLOCK, 0);
        if (current_fd < 0) {
            LILYPAD_ERROR("Failed to create socket: " + std::string(strerror(errno)));
            return false;
        }

        struct sockaddr_un addr{};
        addr.sun_family = AF_UNIX;

        addr.sun_path[0] = '\0';
        memcpy(&addr.sun_path[1], current_path.c_str(), current_path.length());
        socklen_t len = offsetof(struct sockaddr_un, sun_path) + current_path.length() + 1;

        if (connect(current_fd, (struct sockaddr*)&addr, len) == 0) {
            uinput_client_fd_ = current_fd;
            auto* engine = g_lilypad_engine.load(std::memory_order_acquire);
            if (!g_uinput_io_event && engine != nullptr && engine->instance() != nullptr) {
                g_uinput_io_event = engine->instance()->eventLoop().addIOEvent(
                    current_fd,
                    IOEventFlag::In,
                    [](EventSourceIO*, int fd, IOEventFlags) {
                        handleUinputSocketEvent(fd);
                        return true;
                    }
                );
            }
            return true;
        }
        LILYPAD_ERROR("Failed to connect to socket: " + std::string(strerror(errno)));
        g_uinput_io_event.reset();
        int old_fd = uinput_client_fd_.exchange(-1);
        if (old_fd != -1) {
            close(old_fd);
        }
        return false;
    }

    int LilypadState::setup_uinput() {
        return connect_uinput_server() ? uinput_client_fd_.load(std::memory_order_acquire) : -1;
    }

    void LilypadState::send_arm_tap_uinput() const {
        int fd = uinput_client_fd_.load(std::memory_order_acquire);
        if (fd < 0)
            return;
        LilypadIpcPacket pkt{};
        pkt.type = IPC_CMD_ARM_TAP;
        send(fd, &pkt, sizeof(pkt), MSG_NOSIGNAL);
    }

    void LilypadState::send_disarm_tap_uinput() const {
        int fd = uinput_client_fd_.load(std::memory_order_acquire);
        if (fd < 0)
            return;
        LilypadIpcPacket pkt{};
        pkt.type = IPC_CMD_DISARM_TAP;
        send(fd, &pkt, sizeof(pkt), MSG_NOSIGNAL);
    }

    void LilypadState::handleUinputSocketEvent(int fd) {
        LilypadIpcPacket pkt{};
        ssize_t n = recv(fd, &pkt, sizeof(pkt), MSG_DONTWAIT);
        if (n <= 0) {
            if (n == 0 || (errno != EAGAIN && errno != EWOULDBLOCK)) {
                LILYPAD_WARN("Uinput server socket closed or error: " + std::string(strerror(errno)));
                g_uinput_io_event.reset();
                int old_fd = uinput_client_fd_.exchange(-1);
                if (old_fd >= 0) {
                    close(old_fd);
                }
            }
            return;
        }
    }

    void LilypadState::send_backspace_uinput(int count) const {
        if (uinput_client_fd_ < 0 && !connect_uinput_server()) {
            LILYPAD_ERROR("Cannot send backspace since cannot connect to uinput server");
            return;
        }

        LilypadIpcPacket pkt{};
        pkt.type = IPC_CMD_BACKSPACE;
        pkt.code = static_cast<uint32_t>(count);
        ssize_t n = send(uinput_client_fd_, &pkt, sizeof(pkt), MSG_NOSIGNAL);

        if (n < 0) {
            LILYPAD_WARN("Failed to send backspace: " + std::string(strerror(errno)));
            g_uinput_io_event.reset();
            int old_fd = uinput_client_fd_.exchange(-1);
            if (old_fd != -1) {
                close(old_fd);
            }
            if (connect_uinput_server()) {
                LILYPAD_INFO("Reconnected to uinput server successfully");
                send(uinput_client_fd_, &pkt, sizeof(pkt), MSG_NOSIGNAL);
            }
        }

        if (waitAck_) {
            LILYPAD_INFO("Waiting for ack");
            std::this_thread::sleep_for(std::chrono::milliseconds(count * 5));
        }
    }

    bool LilypadState::isAutofillCertain(const SurroundingText& s) {
        if (!s.isValid() || oldPreBuffer_.empty()) {
            return false;
        }

        const auto& text = s.text();
        // Browser URL autofill only exists in single-line inputs (e.g. address/search bar).
        // In multiline editors/chat panels (Monaco, BlockSuite, etc.), any newline signifies
        // multiline text where URL autofill never exists and selection is user/AI text.
        if (text.find('\n') != std::string::npos) {
            return false;
        }

        auto textRange = fcitx::utf8::MakeUTF8CharRange(text);
        std::u32string u32Text(textRange.begin(), textRange.end());
        auto bufRange = fcitx::utf8::MakeUTF8CharRange(oldPreBuffer_);
        std::u32string u32Buf(bufRange.begin(), bufRange.end());

        const unsigned int cursor     = s.cursor();
        const unsigned int anchor     = s.anchor();
        const size_t       u32TextLen = u32Text.length();
        const size_t       buffLen    = u32Buf.length();

        if (buffLen == 0 || cursor > u32TextLen) {
            return false;
        }

        // Bounded Window Search: only look within a narrow window (max 16 chars) immediately preceding the cursor
        // to avoid false positive matches from large pasted text or elsewhere in the document.
        size_t searchStart = (cursor >= buffLen + 8) ? (cursor - buffLen - 8) : 0;
        size_t searchLen   = (cursor + 8 > searchStart) ? (cursor + 8 - searchStart) : 0;
        if (searchStart + searchLen > u32TextLen) {
            searchLen = (u32TextLen > searchStart) ? (u32TextLen - searchStart) : 0;
        }

        std::u32string localWindow = u32Text.substr(searchStart, searchLen);
        size_t localPb = localWindow.find(u32Buf);
        if (localPb == std::u32string::npos) {
            return false;
        }
        size_t pb = searchStart + localPb;

        size_t rangeStart = (cursor >= buffLen) ? (cursor - buffLen) : 0;
        const bool sameprefix = (pb >= rangeStart && pb <= static_cast<size_t>(cursor));

        // Detect browser autofill/autocomplete suggestions via selection.
        if (cursor != anchor) {
            unsigned int selectionStart = std::min(anchor, cursor);
            unsigned int selectionEnd   = std::max(anchor, cursor);

            // Only consider it browser autofill if the selection starts at the cursor
            // and extends to the end of the text (common address bar behavior).
            if (selectionStart >= cursor || (selectionStart < cursor && selectionEnd > cursor)) {
                if (!sameprefix)
                    return false;
                // In single-line URL bar, autofill selection extends to the end of the line
                return selectionEnd == u32TextLen;
            }
        }

        if (u32TextLen == static_cast<size_t>(cursor)) {
            realtextLen.store(u32TextLen, std::memory_order_release);
            return false;
        }

        // Heuristic: rapid text growth in a single-line address bar context.
        if (buffLen > u32TextLen) {
            return false;
        }
        if (u32TextLen > static_cast<size_t>(cursor) + 1 && cursor >= buffLen) {
            if (sameprefix) {
                return true;
            }
        }

        return false;
    }

    bool LilypadState::handleUInputKeyPress(KeyEvent& event, KeySym currentSym, int sleepTime) {
        if (!is_deleting_.load(std::memory_order_acquire)) {
            return false;
        }

        void* active = g_active_deleting_state.load(std::memory_order_acquire);
        if (active != nullptr && active != static_cast<void*>(this)) {
            auto* activeState = static_cast<LilypadState*>(active);
            if (currentSym == FcitxKey_Pause || currentSym == FcitxKey_F24) {
                event.filterAndAccept();
                LILYPAD_INFO("🛡️ [CROSS-CONTEXT] KEY_PAUSE received on secondary context -> routed to active context");
                return activeState->handleUInputKeyPress(event, currentSym, sleepTime);
            }
            if (isBackspace(currentSym)) {
                LILYPAD_INFO("🛡️ [CROSS-CONTEXT] Backspace received on secondary context -> routed to active context");
                return activeState->handleUInputKeyPress(event, currentSym, sleepTime);
            }
            return false;
        }

        if (realMode == LilypadMode::Sequence) {
            if (currentSym == FcitxKey_Pause || currentSym == FcitxKey_F24) {
                event.filterAndAccept();
                auto swallow_now = std::chrono::steady_clock::now();
                uint64_t swallow_duration_us = 0;
                if (uinput_send_time_.time_since_epoch().count() > 0) {
                    swallow_duration_us = static_cast<uint64_t>(
                        std::chrono::duration_cast<std::chrono::microseconds>(swallow_now - uinput_send_time_).count()
                    );
                }
                LILYPAD_INFO("🛡️ [HARDWARE BARRIER KEY] Received KEY_PAUSE hardware barrier sentinel! (swallow=" + std::to_string(swallow_duration_us) + "us)");
                executeCommitSequence(false, swallow_duration_us);
                return true;
            }

            if (isBackspace(currentSym)) {
                if (expected_backspaces_ > 0) {
                    --expected_backspaces_;
                    event.forward();
                    return true; // Passthrough backspaces (1..N) to App so old raw text gets erased!
                }
                // Echo / Extra Backspace during settling window:
                // Forward safely without destroying commit sequence or sequencer state!
                LILYPAD_INFO("🛡️ [UINPUT BS] Extra Backspace echo during settling window. Forwarding safely without abort.");
                event.forward();
                return true;
            }
            return false;
        }

        if (isBackspace(currentSym)) {
            current_backspace_count_ += 1;
            if (current_backspace_count_ < expected_backspaces_) {
                return false; // Allow intermediate backspaces to reach the app to clear autofill/old text.
            }
            std::this_thread::sleep_for(std::chrono::milliseconds(sleepTime));
            // Validate surr cursor pos should match realtextLen after all BS applied
            const auto& surr = ic_->surroundingText();
            if (surr.isValid()) {
                if (surr.cursor() == realtextLen.load(std::memory_order_acquire)) {
                    LILYPAD_INFO("Skip retry");
                } else {
                    // Instantly sync realtextLen with true cursor position instead of blocking main thread
                    realtextLen.store(surr.cursor(), std::memory_order_release);
                }
            }
            ic_->commitString(pending_commit_string_);
            LILYPAD_INFO("Commit: " + pending_commit_string_);
            expected_backspaces_     = 0;
            current_backspace_count_ = 0;
            pending_commit_string_.clear();

            event.filterAndAccept(); // Filter out the final trigger backspace.
            if (g_active_deleting_state.load(std::memory_order_acquire) == static_cast<void*>(this)) {
                g_active_deleting_state.store(nullptr, std::memory_order_release);
            }
            is_deleting_.store(false);
            if (getFrontendName(ic_) == "dbus" && !ic_->surroundingText().isValid())
                replayBufferedKeys();
            return true;
        }
        return false;
    }

    bool LilypadState::checkForwardSpecialKey(KeyEvent& keyEvent, KeySym& currentSym) {
        if (keyEvent.key().isCursorMove() || currentSym == FcitxKey_Tab || currentSym == FcitxKey_KP_Tab || currentSym == FcitxKey_ISO_Left_Tab || currentSym == FcitxKey_Escape ||
            keyEvent.key().hasModifier()) {
            if (watchdog_timer_) {
                watchdog_timer_.reset();
            }
            if (barrier_safety_timer_) {
                barrier_safety_timer_.reset();
            }
            if (commit_timer_) {
                commit_timer_.reset();
            }
            if (settle_timer_) {
                settle_timer_.reset();
            }
            if (disarm_timer_) {
                disarm_timer_.reset();
            }
            if (replay_timer_) {
                replay_timer_.reset();
            }
            is_replaying_ = false;
            commit_triggered_ = false;
            sequencer_.clear();
            if (g_active_deleting_state.load(std::memory_order_acquire) == static_cast<void*>(this)) {
                g_active_deleting_state.store(nullptr, std::memory_order_release);
            }
            is_deleting_.store(false, std::memory_order_release);
            expected_backspaces_     = 0;
            current_backspace_count_ = 0;
            pending_commit_string_.clear();
            hasHistory_ = false;
            if (lilypadEngine_) {
                ResetEngine(lilypadEngine_.handle());
            }
            oldPreBuffer_.clear();
            isPrevSpace_       = false;
            isPrevHyphen_      = false;
            shouldCapitalize_  = false;
            isPrevPunctuation_ = false;
            if (iki_sensor_) {
                iki_sensor_->reset();
            }

            keyEvent.forward();
            return true;
        }

        return false;
    }

    void LilypadState::reset(bool isFocusOut) {
        if (isFocusOut && isUinputMode(realMode.load(std::memory_order_relaxed))) {
            deferredFocusOut_ = true;
            ic_->inputPanel().reset();
            return;
        }
        deferredFocusOut_ = false;
        resetNow(isFocusOut);
    }

    void LilypadState::resetNow(bool isFocusOut) {
        const auto& surrounding = ic_->surroundingText();
        const auto& text        = surrounding.text();
        size_t      textLen     = utf8::length(text);
        realtextLen.store(textLen, std::memory_order_release);
        if (is_deleting_.load(std::memory_order_acquire)) {
            return;
        }

        // Prevent spurious internal input context updates (e.g. ONLYOFFICE / AFFiNE / Electron) from wiping engine buffer
        if (!isFocusOut && isUinputMode(realMode)) {
            return;
        }

        if (lilypadEngine_) {
            isPrevSpace_       = false;
            isPrevHyphen_      = false;
            shouldCapitalize_  = false;
            isPrevPunctuation_ = false;
            if (realMode == LilypadMode::Preedit && isFocusOut) {
                EngineCommitPreedit(lilypadEngine_.handle());
                UniqueCPtr<char> commit(EnginePullCommit(lilypadEngine_.handle()));
                if (commit && (*commit.get() != 0)) {
                    ic_->commitString(commit.get());
                    LILYPAD_INFO("Commit: " + std::string(commit.get()));
                }
            }
            if (isFocusOut) {
                ResetEngine(lilypadEngine_.handle());
                oldPreBuffer_.clear();
                hasHistory_ = false;
                if (iki_sensor_) {
                    iki_sensor_->reset();
                }
            }
        }

        switch (realMode) {
            case LilypadMode::Preedit: {
                ic_->inputPanel().reset();
                ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
                ic_->updatePreedit();
                break;
            }
            case LilypadMode::Sequence:
            case LilypadMode::Minecraft: {
                ic_->inputPanel().reset();
                break;
            }
            default: {
                break;
            }
        }
    }

    void LilypadState::flushDeferredFocusOut() {
        if (!deferredFocusOut_) {
            return;
        }
        deferredFocusOut_ = false;
        resetNow(true);
    }

    void LilypadState::cancelDeferredFocusOut() {
        deferredFocusOut_ = false;
    }

    void LilypadState::clearAllBuffers() {
        LILYPAD_DEBUG("Clear all buffers");
        if (is_deleting_.load(std::memory_order_acquire)) {
            return;
        }
        if (settle_timer_) {
            settle_timer_.reset();
        }
        if (commit_timer_) {
            commit_timer_.reset();
        }
        if (barrier_safety_timer_) {
            barrier_safety_timer_.reset();
        }
        oldPreBuffer_.clear();
        hasHistory_ = false;
        deferredFocusOut_ = false;
        if (!is_deleting_.load(std::memory_order_acquire)) {
            expected_backspaces_     = 0;
            current_backspace_count_ = 0;
            pending_commit_string_.clear();
        }
        emojiBuffer_.clear();
        emojiCandidates_.clear();
        buffered_keys_.clear();
        shouldCapitalize_  = false;
        isPrevSpace_       = false;
        isPrevHyphen_      = false;
        isPrevPunctuation_ = false;
        if (iki_sensor_) {
            iki_sensor_->reset();
        }
        if (lilypadEngine_)
            ResetEngine(lilypadEngine_.handle());
    }

    bool LilypadState::isEmptyHistory() const {
        return !hasHistory_;
    }

    void LilypadState::handleOffModeMacro(KeyEvent& keyEvent, KeySym currentSym) {
        if (checkForwardSpecialKey(keyEvent, currentSym)) {
            keyEvent.forward();
            return;
        }

        if (uinput_client_fd_ < 0) {
            connect_uinput_server();
        }

        if (isBackspace(currentSym)) {
            EngineProcessKeyEvent(lilypadEngine_.handle(), FcitxKey_BackSpace, 0);
            auto preeditC = UniqueCPtr<char>(EnginePullPreedit(lilypadEngine_.handle()));
            oldPreBuffer_ = (preeditC && (*preeditC.get() != 0)) ? preeditC.get() : "";
            keyEvent.forward();
            return;
        }

        if (currentSym == FcitxKey_Return) {
            if (!oldPreBuffer_.empty()) {
                ResetEngine(lilypadEngine_.handle());
                oldPreBuffer_.clear();
            }
            keyEvent.forward();
            return;
        }

        std::string keyUtf8 = Key::keySymToUTF8(currentSym);
        bool        processed = EngineProcessKeyEvent(lilypadEngine_.handle(), currentSym, keyEvent.rawKey().states()) != 0U;

        auto        commitPtr = UniqueCPtr<char>(EnginePullCommit(lilypadEngine_.handle()));
        if (processed && commitPtr && (*commitPtr.get() != 0)) {
            std::string commitStr = commitPtr.get();

            bool isMacroExpansion = false;
            if (keyUtf8.empty()) {
                isMacroExpansion = (commitStr != oldPreBuffer_);
            } else {
                isMacroExpansion = (commitStr != oldPreBuffer_ + keyUtf8);
            }

            if (isMacroExpansion) {
                LILYPAD_INFO("Macro expansion: '" + oldPreBuffer_ + "' -> '" + commitStr + "'");
                if (uinput_client_fd_ >= 0 && !oldPreBuffer_.empty()) {
                    performReplacement(oldPreBuffer_, commitStr);
                } else if (ic_->capabilityFlags().test(CapabilityFlag::SurroundingText)) {
                    const auto& surrounding = ic_->surroundingText();
                    if (surrounding.isValid()) {
                        size_t oldLen = utf8::length(oldPreBuffer_);
                        if (oldLen > 0) {
                            ic_->deleteSurroundingText(-static_cast<int>(oldLen), static_cast<int>(oldLen));
                        }
                        ic_->commitString(commitStr);
                    } else {
                        ic_->commitString(commitStr);
                    }
                } else {
                    ic_->commitString(commitStr);
                }
                keyEvent.filterAndAccept();
            } else {
                keyEvent.forward();
            }

            oldPreBuffer_.clear();
            hasHistory_ = false;
            return;
        }

        if (processed || (commitPtr && (*commitPtr.get() != 0))) {
            auto preeditPtr = UniqueCPtr<char>(EnginePullPreedit(lilypadEngine_.handle()));
            oldPreBuffer_   = (preeditPtr && (*preeditPtr.get() != 0)) ? preeditPtr.get() : "";
            if (!processed) {
                oldPreBuffer_.clear();
                hasHistory_ = false;
            }
            keyEvent.forward();
        } else {
            if (!oldPreBuffer_.empty()) {
                ResetEngine(lilypadEngine_.handle());
                oldPreBuffer_.clear();
            }
            keyEvent.forward();
        }
    }

} // namespace fcitx
