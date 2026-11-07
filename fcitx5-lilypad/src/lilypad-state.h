/*
 * SPDX-FileCopyrightText: 2022-2022 CSSlayer <wengxt@gmail.com>
 * SPDX-FileCopyrightText: 2025-2026 Lý Tất Thành
 * SPDX-FileCopyrightText: 2026 Antigravity
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/**
 * @file lilypad-state.h
 * @brief Input context state management for fcitx5-lilypad.
 */

#ifndef _FCITX5_LILYPAD_STATE_H_
#define _FCITX5_LILYPAD_STATE_H_

#include "lilypad.h"
#include "emoji-entry.h"
#include "lilypad-config.h"
#include "lilypad-sequencer.h"
#include "iki-sensors/iki-sensor.h"
#include "iki-sensors/iki-sensor-factory.h"

#include <cstddef>
#include <fcitx-utils/misc.h>
#include <fcitx-utils/event.h>
#include <fcitx/inputcontext.h>

struct EmojiEntry;

namespace fcitx {
    class LilypadEngine;
    class CommonCandidateList;
    class SurroundingText;

    /**
     * @brief Per-input-context state for Lilypad input method.
     *
     * Manages the input state, buffers, and mode-specific handling for each input context.
     */
    class LilypadState final : public InputContextProperty {
      public:
        /**
         * @brief Constructs a new state instance.
         * @param engine Pointer to the Lilypad engine.
         * @param ic Pointer to the input context.
         */
        LilypadState(LilypadEngine* engine, InputContext* ic);
        ~LilypadState() override;

        /**
         * @brief Initializes the bamboo engine for this state.
         */
        void setEngine();

        /**
         * @brief Applies current options to the engine.
         */
        void setOption();

        /**
         * @brief Main key event handler.
         * @param keyEvent The key event to process.
         */
        void keyEvent(KeyEvent& keyEvent);

        /**
         * @brief Resets the input state.
         * @param isFocusOut If true, indicates the reset is due to a focus-out event, which may trigger committing the preedit text.
         */
        void reset(bool isFocusOut = false);

        /**
         * @brief Executes immediate reset logic.
         * @param isFocusOut If true, indicates the reset is due to a focus-out event.
         */
        void resetNow(bool isFocusOut = false);

        /**
         * @brief Flushes deferred focus out if pending.
         */
        void flushDeferredFocusOut();

        /**
         * @brief Cancels deferred focus out when a focus bounce is detected.
         */
        void cancelDeferredFocusOut();

        /**
         * @brief Records a Focus Bounce event to inform the dynamic DOM lag governor.
         */
        void recordFocusBounce();

        /**
         * @brief Commits the current buffer.
         */
        void commitBuffer();

        /**
         * @brief Clears all internal buffers.
         */
        void clearAllBuffers();

        /**
         * @brief Checks if history buffer is empty.
         * @return True if no history.
         */
        bool isEmptyHistory() const;
        friend class EmojiCandidateWord;
        friend class LilypadEngine;

      private:
        static constexpr size_t MAX_BUFFERED_KEYS = 50;

        LilypadEngine*            engine_;
        InputContext*           ic_;
        CGoObject               lilypadEngine_;
        std::string             oldPreBuffer_;
        bool                    hasHistory_              = false;
        int                     expected_backspaces_     = 0;
        int                     current_backspace_count_ = 0;
        std::string             pending_commit_string_;
        std::string             emojiBuffer_;
        std::vector<EmojiEntry> emojiCandidates_;
        bool                    waitAck_ = false;
        std::vector<KeyEntry>   buffered_keys_; ///< Keystrokes buffered during replacement
        DedupRingBuffer         dedup_buffer_;  ///< Bi-directional deduplication ring buffer
        bool                    isPrevSpace_        = false;
        bool                    isPrevHyphen_       = false;
        bool                    shouldCapitalize_   = false;
        bool                    isPrevPunctuation_  = false;
        int64_t                 lastDeactivateTime_ = 0;
        bool                    deferredFocusOut_   = false;
        bool                    wa_chromium_flag    = false;
        bool                    is_replaying_       = false; ///< Strict FIFO lock: active when replaying buffered keys
        Sequencer               sequencer_;
        std::unique_ptr<EventSourceTime> commit_timer_;
        std::unique_ptr<EventSourceTime> settle_timer_;
        std::unique_ptr<EventSourceTime> disarm_timer_; ///< Grace disarm timer for hardware tap
        std::unique_ptr<EventSourceTime> replay_timer_; ///< Micro-tick pacing timer for replaying buffered keys
        std::unique_ptr<EventSourceTime> watchdog_timer_; ///< Hard timeout (300ms) emergency watchdog timer
        std::unique_ptr<EventSourceTime> barrier_safety_timer_; ///< Phase 8.4: Barrier safety timer (20ms) for KEY_F24 sentinel
        std::unique_ptr<EventSourceTime> focus_settled_timer_; ///< Phase 8.5: Focus Settled Debounce timer for Chromium DOM reflow
        std::chrono::steady_clock::time_point uinput_send_time_{}; ///< Timestamp when uinput backspaces were emitted

        // Phase 8.4: Hardware Barrier Key KEY_F24 & Safety Timer
        bool                                  commit_triggered_ = false;
        int                                   original_bs_count_ = 0;

        // Phase 8.3: Dynamic Heavy-DOM & Per-Tab Governor
        uint64_t                              last_swallow_duration_us_ = 0;
        std::chrono::steady_clock::time_point last_focus_bounce_time_{};
        bool                                  has_recent_focus_bounce_ = false;

        /**
         * @brief Phase 8.5: Schedules a debounced commit when Chromium focus bounces settle.
         * @param debounce_us Microseconds of quietude required after the last focus bounce.
         */
        void scheduleFocusSettledCommit(uint64_t debounce_us);

        /**
         * @brief Phase 8.4: Executes commit sequence when KEY_F24 arrives or barrier safety timer fires.
         * @param from_safety_timer True if triggered by 20ms safety timer rescue.
         * @param swallow_duration_us Measured or fallback roundtrip latency in microseconds.
         */
        void executeCommitSequence(bool from_safety_timer, uint64_t swallow_duration_us);

        /**
         * @brief Checks if the current tab/context is experiencing heavy DOM or event-loop lag.
         */
        bool isTabLagging() const;

        /**
         * @brief Calculates dynamic pre-commit settling delay based on runtime DOM load.
         */
        uint64_t calculateDynamicSettlingDelayUs(uint64_t micro_delay_us, uint64_t swallow_duration_us);

        /**
         * @brief Calculates dynamic post-commit settling window based on runtime DOM load.
         */
        uint64_t calculateDynamicPostSettlingUs() const;

        /**
         * @brief Connects to the uinput server.
         * @return True if connection successful.
         */
        static bool connect_uinput_server();

        /**
         * @brief Sets up uinput device.
         * @return File descriptor or -1 on error.
         */
        static int setup_uinput();

        /**
         * @brief Sends backspace key events via uinput.
         * @param count Number of backspaces to send.
         */
        void send_backspace_uinput(int count) const;

        /**
         * @brief Checks if autofill is certain for surrounding text.
         * @param s The surrounding text.
         * @return True if autofill should proceed.
         */
        bool isAutofillCertain(const SurroundingText& s);

        /**
         * @brief Handles key events in preedit mode.
         * @param keyEvent The key event to process.
         * @param currentSym Current key symbol.
         */
        void handlePreeditMode(KeyEvent& keyEvent, KeySym currentSym);

        /**
         * @brief Updates emoji page status in candidate list.
         * @param commonList The candidate list to update.
         */
        void updateEmojiPageStatus(CommonCandidateList* commonList);

        /**
         * @brief Handles key events in emoji mode.
         * @param keyEvent The key event to process.
         */
        void handleEmojiMode(KeyEvent& keyEvent);

        /**
         * @brief Updates preedit display for emoji mode.
         */
        void updateEmojiPreedit();

        /**
         * @brief Handles key press in uinput mode.
         * @param event The key event.
         * @param currentSym Current key symbol.
         * @param sleepTime Delay in microseconds.
         * @return True if event was handled.
         */
        bool handleUInputKeyPress(KeyEvent& event, KeySym currentSym, int sleepTime);

        /**
         * @brief Performs text replacement via uinput.
         * @param deletedPart Text to delete.
         * @param addedPart Text to insert.
         */
        void performReplacement(const std::string& deletedPart, const std::string& addedPart);

        /**
         * @brief Handles the double space to period replacement.
         */
        void handleDoubleSpaceReplacement();

        /**
         * @brief Handles the double hyphen to em-dash replacement.
         */
        void handleDoubleHyphenReplacement();

        /**
         * @brief Checks and forwards special keys.
         * @param keyEvent The key event.
         * @param currentSym Current key symbol (may be modified).
         * @return True if key was forwarded.
         */
        bool checkForwardSpecialKey(KeyEvent& keyEvent, KeySym& currentSym);

        /**
         * @brief Handles uinput mode processing.
         * @param keyEvent The key event.
         * @param currentSym Current key symbol.
         * @param sleepTime Delay in microseconds.
         */
        void handleUinputMode(KeyEvent& keyEvent, KeySym currentSym);

        /**
         * @brief Handles surrounding text mode.
         * @param keyEvent The key event.
         * @param currentSym Current key symbol.
         */
        void handleSurroundingText(KeyEvent& keyEvent, KeySym currentSym);

        /**
         * @brief Handles Off mode with macro shadow processing.
         * @param keyEvent The key event.
         * @param currentSym Current key symbol.
         */
        void handleOffModeMacro(KeyEvent& keyEvent, KeySym currentSym);

        /**
         * @brief Handles processing normal key events.
         * @param keyEvent The key event.
         * @param currentSym Current key symbol.
         */
        void processNormalKey(KeyEvent& keyEvent, KeySym currentSym);

        /**
         * @brief Replays keystrokes buffered during replacement.
         *
         * When is_deleting_ is true, non-special keystrokes are buffered
         * instead of being discarded. This method replays them after the
         * replacement completes.
         */
        void replayBufferedKeys();

        /**
         * @brief Emergency recovery when hard timeout (300ms) expires during replacement.
         *
         * Resets internal word buffers and forwards all queued keystrokes as raw input.
         */
        void purgeContextEmergency();

        /**
         * @brief Explicitly arms hardware tap in uinput server.
         */
        void send_arm_tap_uinput() const;

        /**
         * @brief Explicitly disarms hardware tap in uinput server.
         */
        void send_disarm_tap_uinput() const;

        /**
         * @brief Handles incoming socket events from uinput server.
         * @param fd Socket file descriptor.
         */
        static void handleUinputSocketEvent(int fd);

        // IKI (Inter-Keystroke Interval) Modular Sensor
        std::unique_ptr<IIkiSensor> iki_sensor_; ///< Dedicated IKI Sensor Module

        /**
         * @brief Gets the current IKI sensor instance.
         */
        IIkiSensor* ikiSensor() const { return iki_sensor_.get(); }

        /**
         * @brief Gets the current smoothed IKI estimate in milliseconds.
         * @return Smoothed IKI in ms.
         */
        uint64_t getIkiMs() const { return iki_sensor_ ? iki_sensor_->get_ema_iki_ms() : 150; }
    };

} // namespace fcitx

#endif // _FCITX5_LILYPAD_STATE_H_
