/*
 * SPDX-FileCopyrightText: 2022-2022 CSSlayer <wengxt@gmail.com>
 * SPDX-FileCopyrightText: 2025-2026 Lý Tất Thành
 * SPDX-FileCopyrightText: 2026 Antigravity
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/**
 * @file lilypad-state.cpp
 * @brief Core Lifecycle, Bamboo Core Engine Interface, Key Event Dispatcher & Candidate UI.
 */

#include "lilypad-state.h"
#include "lilypad-engine.h"
#include "lilypad-candidates.h"
#include "lilypad-utils.h"
#include "lilypad.h"

#include <cstddef>
#include <fcitx-utils/log.h>
#include <fcitx-utils/utf8.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputpanel.h>
#include <fcitx/menu.h>
#include <fcitx/userinterface.h>

#include <algorithm>
#include <string>
#include <vector>

#include "ack-sensors/sensor-factory.h"

namespace fcitx {
    constexpr int MAX_SCAN_LENGTH = 15;

    static inline bool isWordBreak(uint32_t ucs4) {
        // Space, tab, newline, carriage return, null, or punctuation/symbols (: ; < = > ? @)
        return ucs4 == ' ' || ucs4 == '\t' || ucs4 == '\n' || ucs4 == '\r' || ucs4 == 0 || (ucs4 >= 58 && ucs4 <= 64);
    }

    LilypadState::LilypadState(LilypadEngine* engine, InputContext* ic) : engine_(engine), ic_(ic) {
        sequencer_.set_sensor(AckSensorFactory::create_sensor());
        iki_sensor_ = IkiSensorFactory::create_sensor(
            *engine_->config().enableIkiAdaptive,
            *engine_->config().ikiMinMs,
            *engine_->config().ikiMaxMs
        );
        setEngine();
    }

    LilypadState::~LilypadState() {
        if (g_active_deleting_state.load(std::memory_order_acquire) == static_cast<void*>(this)) {
            g_active_deleting_state.store(nullptr, std::memory_order_release);
            is_deleting_.store(false, std::memory_order_release);
        }
    }

    void LilypadState::setEngine() {
        lilypadEngine_.reset();
        realMode = engine_->config().mode.value();

        if (engine_->config().inputMethod.value() == "Custom") {
            const auto&        keymaps = *engine_->customKeymap().customKeymap;
            std::vector<char*> charArray;
            charArray.reserve((keymaps.size() * 2) + 1);
            for (const auto& keymap : keymaps) {
                charArray.push_back(const_cast<char*>(keymap.key->data()));   //NOLINT
                charArray.push_back(const_cast<char*>(keymap.value->data())); //NOLINT
            }
            charArray.push_back(nullptr);
            lilypadEngine_.reset(NewCustomEngine(charArray.data(), engine_->dictionary(), engine_->macroTable()));
        } else {
            lilypadEngine_.reset(NewEngine(engine_->config().inputMethod->data(), engine_->dictionary(), engine_->macroTable()));
        }
        setOption();
    }

    void LilypadState::setOption() {
        if (!lilypadEngine_)
            return;
        FcitxBambooEngineOption option = {
            .autoNonVnRestore    = *engine_->config().autoNonVnRestore,
            .ddFreeStyle         = *engine_->config().ddFreeStyle,
            .macroEnabled        = *engine_->config().enableMacro,
            .autoCapitalizeMacro = *engine_->config().capitalizeMacro,
            .spellCheckWithDicts = *engine_->config().spellCheck,
            .outputCharset       = engine_->config().outputCharset->data(),
            .modernStyle         = *engine_->config().modernStyle,
            .freeMarking         = *engine_->config().freeMarking,
            .w2u                 = static_cast<int>(*engine_->config().w2u),
            .bracketTransform    = static_cast<int>(*engine_->config().bracketTransform),
            .timeFormat          = engine_->config().timeFormat->data(),
            .dateFormat          = engine_->config().dateFormat->data(),
        };

        EngineSetOption(lilypadEngine_.handle(), &option);
    }

    void LilypadState::handlePreeditMode(KeyEvent& keyEvent, KeySym currentSym) {
        if (checkForwardSpecialKey(keyEvent, currentSym)) {
            ic_->inputPanel().reset();
            ic_->updatePreedit();
            ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
            keyEvent.forward();
            return;
        }

        if (isBackspace(currentSym)) {
            UniqueCPtr<char> preeditCheck(EnginePullPreedit(lilypadEngine_.handle()));
            if (!preeditCheck || (*preeditCheck.get() == 0)) {
                // No active preedit: physical Backspace must be forwarded to application to delete space/text!
                ResetEngine(lilypadEngine_.handle());
                ic_->inputPanel().reset();
                ic_->updatePreedit();
                ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
                keyEvent.forward();
                return;
            }
        }

        if (currentSym == FcitxKey_Return) {
            UniqueCPtr<char> preeditCheck(EnginePullPreedit(lilypadEngine_.handle()));
            if (preeditCheck && (*preeditCheck.get() != 0)) {
                EngineCommitPreedit(lilypadEngine_.handle());
                auto commit = UniqueCPtr<char>(EnginePullCommit(lilypadEngine_.handle()));
                ic_->inputPanel().reset();
                ic_->updatePreedit();
                if (commit && (*commit.get() != 0)) {
                    ic_->commitString(commit.get());
                }
                ResetEngine(lilypadEngine_.handle());
                ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
                keyEvent.filterAndAccept();
                return;
            }
            keyEvent.forward();
            return;
        }

        bool processed = EngineProcessKeyEvent(lilypadEngine_.handle(), currentSym, keyEvent.rawKey().states()) != 0U;

        auto commit = UniqueCPtr<char>(EnginePullCommit(lilypadEngine_.handle()));
        if (commit && (*commit.get() != 0)) {
            LILYPAD_INFO("Commit: " + std::string(commit.get()));
            // Phase: Reset preedit in GTK4 / Client FIRST before committing string to prevent buffer scramble
            ic_->inputPanel().reset();
            ic_->updatePreedit();
            ic_->commitString(commit.get());
        }

        UniqueCPtr<char> preedit(EnginePullPreedit(lilypadEngine_.handle()));
        if (preedit && (*preedit.get() != 0)) {
            std::string_view view = preedit.get();
            Text             text;
            TextFormatFlags  fmt = TextFormatFlag::Underline;
            if (utf8::validate(view))
                text.append(std::string(view), fmt);
            text.setCursor(static_cast<int>(text.textLength()));
            if (ic_->capabilityFlags().test(CapabilityFlag::Preedit))
                ic_->inputPanel().setClientPreedit(text);
            else
                ic_->inputPanel().setPreedit(text);
        } else {
            ic_->inputPanel().reset();
            ic_->inputPanel().setClientPreedit(Text());
            ic_->inputPanel().setPreedit(Text());
        }
        ic_->updatePreedit();
        ic_->updateUserInterface(UserInterfaceComponent::InputPanel);

        if (processed || (commit && *commit.get() != 0)) {
            keyEvent.filterAndAccept();
        } else {
            keyEvent.forward();
        }
    }

    void LilypadState::handleUinputMode(KeyEvent& keyEvent, KeySym currentSym) {
        if (checkForwardSpecialKey(keyEvent, currentSym)) {
            keyEvent.forward();
            return;
        }

        if (uinput_client_fd_ < 0) {
            setup_uinput();
        }

        if (isBackspace(currentSym) || currentSym == FcitxKey_Return) {
            if (isBackspace(currentSym)) {
                hasHistory_ = true;
                EngineProcessKeyEvent(lilypadEngine_.handle(), FcitxKey_BackSpace, 0);
                UniqueCPtr<char> preeditC(EnginePullPreedit(lilypadEngine_.handle()));
                oldPreBuffer_ = (preeditC && (*preeditC.get() != 0)) ? preeditC.get() : "";
            } else {
                hasHistory_ = false;
                ResetEngine(lilypadEngine_.handle());
                oldPreBuffer_.clear();
            }
            keyEvent.forward();
            return;
        }

        std::string keyUtf8 = Key::keySymToUTF8(currentSym);
        if (keyUtf8.empty()) {
            keyEvent.forward();
            return;
        }

        bool processed = EngineProcessKeyEvent(lilypadEngine_.handle(), currentSym, keyEvent.rawKey().states()) != 0U;

        auto commitF = UniqueCPtr<char>(EnginePullCommit(lilypadEngine_.handle()));
        if (commitF && (*commitF.get() != 0)) {
            std::string commitStr = commitF.get();
            std::string deletedPart;
            std::string addedPart;
            compareAndSplitStrings(oldPreBuffer_, commitStr, deletedPart, addedPart);

            if (!deletedPart.empty()) {
                performReplacement(deletedPart, addedPart);
                keyEvent.filterAndAccept();
            } else {
                bool wasAutoCapitalized = (currentSym != keyEvent.rawKey().sym());
                if (!addedPart.empty() && (keyUtf8 != addedPart || wasAutoCapitalized)) {
                    if (addedPart.size() > 1 && addedPart.back() == ' ') {
#if __cplusplus >= 202002L
                        addedPart.resize(addedPart.size() - 1);
#else
                        addedPart = addedPart.substr(0, addedPart.size() - 1);
#endif
                    }
                    ic_->commitString(addedPart);
                    LILYPAD_INFO("Commit: " + addedPart);
                    keyEvent.filterAndAccept();
                } else {
                    keyEvent.forward();
                }
            }

            hasHistory_ = false;
            ResetEngine(lilypadEngine_.handle());
            oldPreBuffer_.clear();

            return;
        }

        if (!processed) {
            UniqueCPtr<char> preeditC(EnginePullPreedit(lilypadEngine_.handle()));
            if (!preeditC || (*preeditC.get() == 0)) {
                hasHistory_ = false;
                ResetEngine(lilypadEngine_.handle());
                oldPreBuffer_.clear();
                keyEvent.forward();
            }
            return;
        }

        hasHistory_ = true;
        realtextLen.fetch_add(1, std::memory_order_acq_rel);

        UniqueCPtr<char> preeditC(EnginePullPreedit(lilypadEngine_.handle()));
        std::string      preeditStr = (preeditC && (*preeditC.get() != 0)) ? preeditC.get() : "";

        std::string      deletedPart;
        std::string      addedPart;

        if (compareAndSplitStrings(oldPreBuffer_, preeditStr, deletedPart, addedPart) != 0) {
            if (deletedPart.empty()) {
                bool isCommit           = false;
                bool wasAutoCapitalized = (currentSym != keyEvent.rawKey().sym());
                if (!addedPart.empty()) {
                    oldPreBuffer_ = preeditStr;
                    if (wasAutoCapitalized || addedPart != keyUtf8) {
                        ic_->commitString(addedPart);
                        LILYPAD_INFO("Commit: " + addedPart);
                        keyEvent.filterAndAccept();
                        isCommit = true;
                    }
                }
                if (!isCommit) {
                    keyEvent.forward();
                }
            } else {
                if (uinput_client_fd_ < 0) {
                    LILYPAD_ERROR("Cannot connect to uinput server, commit rawkey");
                    std::string rawKey = keyEvent.key().toString();
                    if (!rawKey.empty()) {
                        ic_->commitString(rawKey);
                    }
                    return;
                }

                keyEvent.filterAndAccept();
                performReplacement(deletedPart, addedPart);
                oldPreBuffer_ = preeditStr;
            }
        }
    }

    void LilypadState::handleSurroundingText(KeyEvent& keyEvent, KeySym currentSym) {
        if (checkForwardSpecialKey(keyEvent, currentSym)) {
            keyEvent.forward();
            return;
        }
        auto* ic = keyEvent.inputContext();
        if ((ic == nullptr) || !ic->capabilityFlags().test(CapabilityFlag::SurroundingText)) {
            LILYPAD_WARN("Surrounding text not supported");
            keyEvent.forward();
            return;
        }

        const auto& surrounding = ic->surroundingText();
        if (!surrounding.isValid()) {
            LILYPAD_WARN("Surrounding text is invalid");
            keyEvent.forward();
            return;
        }

        if (isBackspace(keyEvent.rawKey().sym())) {
            ResetEngine(lilypadEngine_.handle());
            keyEvent.forward();
            return;
        }

        const std::string& text   = surrounding.text();
        unsigned int       cursor = std::min(surrounding.anchor(), surrounding.cursor());
        size_t             textLen = utf8::lengthValidated(text);

        if (textLen == utf8::INVALID_LENGTH || cursor <= 0 || cursor > textLen) {
            processNormalKey(keyEvent, currentSym);
            return;
        }

        {
            auto startIter = utf8::nextNChar(text.begin(), cursor);
            auto endIter   = startIter;

            int  scanCount = 0;
            while (startIter != text.begin() && scanCount < MAX_SCAN_LENGTH) {
                auto prev = startIter;
                if (prev != text.begin()) {
                    --prev;
                    while (prev != text.begin() && ((*prev & 0xC0) == 0x80)) {
                        --prev;
                    }
                }

                uint32_t ucs4 = utf8::getChar(prev, text.end());
                if (isWordBreak(ucs4))
                    break;

                startIter = prev;
                ++scanCount;
            }

            std::string oldWord(startIter, endIter);

            if (oldWord.empty()) {
                processNormalKey(keyEvent, currentSym);
                return;
            }

            EngineRebuildFromText(lilypadEngine_.handle(), oldWord.c_str());

            bool processed = EngineProcessKeyEvent(lilypadEngine_.handle(), currentSym, keyEvent.rawKey().states()) != 0U;

            if (!processed) {
                keyEvent.forward();
                ResetEngine(lilypadEngine_.handle());
                return;
            }

            auto        commitPtr  = UniqueCPtr<char>(EnginePullCommit(lilypadEngine_.handle()));
            auto        preeditPtr = UniqueCPtr<char>(EnginePullPreedit(lilypadEngine_.handle()));

            std::string newWord;
            if (commitPtr && (*commitPtr.get() != 0))
                newWord += commitPtr.get();
            if (preeditPtr && (*preeditPtr.get() != 0))
                newWord += preeditPtr.get();

            std::string deletedPart;
            std::string addedPart;
            compareAndSplitStrings(oldWord, newWord, deletedPart, addedPart);
            if ((deletedPart.empty() || deletedPart == oldWord) && addedPart == keyEvent.key().toString()) {
                ResetEngine(lilypadEngine_.handle());
                keyEvent.forward();
                return;
            }

            if (!deletedPart.empty() || !addedPart.empty()) {
                size_t charsToDelete = utf8::length(deletedPart);

                if (charsToDelete > 0) {
                    ic->deleteSurroundingText(-static_cast<int>(charsToDelete), static_cast<int>(charsToDelete));
                }

                if (!addedPart.empty()) {
                    ic->commitString(addedPart);
                    LILYPAD_INFO("Commit: " + addedPart);
                }

                ResetEngine(lilypadEngine_.handle());
                keyEvent.filterAndAccept();
                return;
            }

            ResetEngine(lilypadEngine_.handle());
            keyEvent.filterAndAccept();
            return;
        }
    }

    void LilypadState::processNormalKey(KeyEvent& keyEvent, KeySym currentSym) {
        auto* ic = keyEvent.inputContext();
        ResetEngine(lilypadEngine_.handle());
        bool processed = EngineProcessKeyEvent(lilypadEngine_.handle(), currentSym, keyEvent.rawKey().states()) != 0U;
        if (processed) {
            auto        commitPtr  = UniqueCPtr<char>(EnginePullCommit(lilypadEngine_.handle()));
            auto        preeditPtr = UniqueCPtr<char>(EnginePullPreedit(lilypadEngine_.handle()));
            std::string out;
            if (commitPtr && (*commitPtr.get() != 0))
                out += commitPtr.get();
            if (preeditPtr && (*preeditPtr.get() != 0))
                out += preeditPtr.get();

            if (!out.empty()) {
                LILYPAD_INFO("Commit: " + out);
                ic->commitString(out);
            }

            ResetEngine(lilypadEngine_.handle());
            keyEvent.filterAndAccept();
        } else {
            keyEvent.forward();
        }
    }

    void LilypadState::handleDoubleSpaceReplacement() {
        performReplacement(" ", ". ");
        LILYPAD_INFO("Commit: . ");
        if (*engine_->config().autoCapitalizeAfterPunctuation) {
            isPrevPunctuation_ = true;
            shouldCapitalize_  = true;
        }
    }

    void LilypadState::handleDoubleHyphenReplacement() {
        // Em-dash (U+2014)
        std::string emDash = "—";
        performReplacement("-", emDash);
        LILYPAD_INFO("Commit: — (em-dash)");
    }

    void LilypadState::keyEvent(KeyEvent& keyEvent) {
        if (!lilypadEngine_ || keyEvent.isRelease() || keyEvent.rawKey().isModifier())
            return;
        if (uinput_client_fd_ < 0) {
            LILYPAD_WARN("Cannot connect to uinput server, reconnecting....");
            connect_uinput_server();
        }

        if (needEngineReset.load()) {
            LILYPAD_INFO("Need engine reset");
            oldPreBuffer_.clear();
            hasHistory_ = false;
            ResetEngine(lilypadEngine_.handle());
            is_deleting_.store(false);
            current_backspace_count_ = 0;
            isPrevSpace_             = false;
            shouldCapitalize_        = false;
            isPrevPunctuation_       = false;
            needEngineReset.store(false);
            dedup_buffer_.clear();
        }

        if (g_mouse_clicked.load(std::memory_order_acquire) && !is_deleting_.load(std::memory_order_acquire)) {
            g_mouse_clicked.store(false, std::memory_order_release);
            clearAllBuffers();
            dedup_buffer_.clear();
        }
        KeySym currentSym = keyEvent.rawKey().sym();
        bool isSentinelKey = (currentSym == FcitxKey_Pause || currentSym == FcitxKey_F24);
        bool isAutomatedBackspace = is_deleting_.load(std::memory_order_acquire) && (isBackspace(currentSym) || isSentinelKey);

        if (iki_sensor_) {
            iki_sensor_->on_key_event(std::chrono::steady_clock::now(), isAutomatedBackspace);
        }

        if (*engine_->config().autoCapitalizeAfterPunctuation && !isAutomatedBackspace) {
            if (shouldCapitalize_) {
                if (currentSym >= FcitxKey_a && currentSym <= FcitxKey_z) {
                    currentSym = static_cast<KeySym>(currentSym - (FcitxKey_a - FcitxKey_A));
                    keyEvent.setKey(Key(currentSym, keyEvent.rawKey().states()));
                    shouldCapitalize_ = false;
                } else if (currentSym != FcitxKey_space) {
                    shouldCapitalize_ = false;
                }
            }
            switch (currentSym) {
                case FcitxKey_period: case FcitxKey_exclam: case FcitxKey_question:
                    isPrevPunctuation_ = true; break;
                case FcitxKey_Return: case FcitxKey_KP_Enter:
                    shouldCapitalize_ = true; isPrevPunctuation_ = false; break;
                case FcitxKey_space:
                    if (isPrevPunctuation_) { shouldCapitalize_ = true; isPrevPunctuation_ = false; }
                    break;
                default:
                    if (currentSym != FcitxKey_space) isPrevPunctuation_ = false;
                    break;
            }
        }

        if (checkForwardSpecialKey(keyEvent, currentSym)) {
            return;
        }

        // Align realtextLen to actual cursor position whenever starting a new word or idle
        if (oldPreBuffer_.empty() && ic_ && ic_->surroundingText().isValid()) {
            realtextLen.store(ic_->surroundingText().cursor(), std::memory_order_release);
        }

        if (isSentinelKey) {
            // Phase 8.4: Non-destructive hardware barrier sentinel KEY_PAUSE / KEY_F24
            if (is_deleting_.load(std::memory_order_acquire)) {
                handleUInputKeyPress(keyEvent, currentSym, 0);
            } else {
                // Arrived late after safety timer or watchdog already released the barrier.
                // Filter and swallow silently with diagnostic log.
                LILYPAD_INFO("🛡️ [HARDWARE BARRIER KEY] Received late sentinel key (Pause/F24) after barrier released. Swallowed safely.");
                keyEvent.filterAndAccept();
            }
            return;
        }

        bool queue_busy = is_deleting_.load(std::memory_order_acquire) || is_replaying_ || !buffered_keys_.empty();
        if (queue_busy) {
            if (is_deleting_.load(std::memory_order_acquire)) {
                if (realMode == LilypadMode::Sequence) {
                    if (sequencer_.is_hard_timeout()) {
                        LILYPAD_WARN("⚠️ [HARD TIMEOUT TRIGGERED] Elapsed time exceeded 300ms during keyEvent. Purging emergency!");
                        purgeContextEmergency();
                        processNormalKey(keyEvent, currentSym);
                        return;
                    }
                    if (sequencer_.is_soft_timeout(getIkiMs()) && sequencer_.barrier_state() != BarrierState::AppLagHolding) {
                        sequencer_.set_app_lag_holding();
                        LILYPAD_INFO("🛡️ [SOFT TIMEOUT] App latency spike detected, holding keys in RAM buffer");
                    }
                }

                if (isBackspace(currentSym)) {
                    if (realtextLen.load(std::memory_order_acquire) > 0)
                        realtextLen.fetch_sub(1, std::memory_order_acq_rel);
                    if (handleUInputKeyPress(keyEvent, currentSym, 20)) {
                        return;
                    }
                } else {
                    std::string keyUtf8Check = Key::keySymToUTF8(currentSym);
                    if (!keyUtf8Check.empty()) {
                        if (buffered_keys_.size() < MAX_BUFFERED_KEYS) {
                            LILYPAD_WARN("Typing so fast, add key to queue");
                            buffered_keys_.push_back({.sym = currentSym, .state = keyEvent.rawKey().states(), .enter_time = std::chrono::steady_clock::now()});
                        }
                    }
                    keyEvent.filterAndAccept();
                }
                return;
            }

            // Strict FIFO Lock: Replay queue is draining, enqueue new keys behind pending replay keys
            std::string keyUtf8Check = Key::keySymToUTF8(currentSym);
            if (!keyUtf8Check.empty() || isBackspace(currentSym)) {
                if (buffered_keys_.size() < MAX_BUFFERED_KEYS) {
                    LILYPAD_INFO("🛡️ [FIFO ORDERING] Replay active: queuing key behind pending replay keys");
                    buffered_keys_.push_back({.sym = currentSym, .state = keyEvent.rawKey().states(), .enter_time = std::chrono::steady_clock::now()});
                }
            }
            if (!replay_timer_) {
                auto& loop = engine_->instance()->eventLoop();
                replay_timer_ = loop.addTimeEvent(CLOCK_MONOTONIC, ::fcitx::now(CLOCK_MONOTONIC), 1, [this](EventSourceTime*, uint64_t) {
                    replayBufferedKeys();
                    return false;
                });
            }
            keyEvent.filterAndAccept();
            return;
        }

        if (*engine_->config().doubleSpaceToPeriod) {
            bool isSpaceKey = (currentSym == FcitxKey_space || currentSym == FcitxKey_KP_Space);
            if (isSpaceKey && !keyEvent.key().hasModifier()) {
                if (isPrevSpace_) {
                    keyEvent.filterAndAccept();
                    handleDoubleSpaceReplacement();
                    isPrevSpace_ = false;
                    return;
                }
                isPrevSpace_ = true;
            } else { isPrevSpace_ = false; }
        }

        if (*engine_->config().doubleHyphenToEmDash) {
            bool isHyphenKey = (currentSym == FcitxKey_minus || currentSym == FcitxKey_KP_Subtract);
            if (isHyphenKey && !keyEvent.key().hasModifier()) {
                if (isPrevHyphen_) {
                    keyEvent.filterAndAccept();
                    handleDoubleHyphenReplacement();
                    isPrevHyphen_ = false;
                    return;
                }
                isPrevHyphen_ = true;
            } else { isPrevHyphen_ = false; }
        }

        switch (realMode) {
            case LilypadMode::Sequence:
            case LilypadMode::Minecraft: {
                handleUinputMode(keyEvent, currentSym);
                break;
            }
            case LilypadMode::Preedit: {
                handlePreeditMode(keyEvent, currentSym);
                break;
            }
            default: {
                break;
            }
        }
    }

    void LilypadState::commitBuffer() {
        switch (realMode) {
            case LilypadMode::Preedit: {
                ic_->inputPanel().reset();
                ic_->updatePreedit();
                if (lilypadEngine_) {
                    EngineCommitPreedit(lilypadEngine_.handle());
                    UniqueCPtr<char> commit(EnginePullCommit(lilypadEngine_.handle()));
                    if (commit && (*commit.get() != 0))
                        ic_->commitString(commit.get());
                    ResetEngine(lilypadEngine_.handle());
                }
                ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
                break;
            }
            case LilypadMode::Sequence:
            case LilypadMode::Minecraft: {
                if (lilypadEngine_) {
                    ResetEngine(lilypadEngine_.handle());
                }
                break;
            }
            default: {
                break;
            }
        }
    }

} // namespace fcitx
