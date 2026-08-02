/*
 * SPDX-FileCopyrightText: 2026 Nguyễn Hoàng Kỳ  <nhktmdzhg@gmail.com>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */
#include "lilypad-candidates.h"
#include "lilypad-state.h"
#include "lilypad-engine.h"

#include <fcitx/inputpanel.h>

namespace fcitx {

    // EmojiCandidateWord implementation
    EmojiCandidateWord::EmojiCandidateWord(Text text, LilypadState* state, EmojiEntry entry) : CandidateWord(std::move(text)), state_(state), entry_(std::move(entry)) {}

    void EmojiCandidateWord::select(InputContext* /*inputContext*/) const {
        state_->ic_->commitString(entry_.output);
        LILYPAD_INFO("Emoji committed: " + entry_.output);

        state_->engine_->emojiLoader().recordHistory(entry_);

        state_->emojiBuffer_.clear();
        state_->emojiCandidates_.clear();

        state_->ic_->inputPanel().reset();
        state_->ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
        state_->updateEmojiPreedit();
    }

    // AppModeCandidateWord implementation
    AppModeCandidateWord::AppModeCandidateWord(Text text, std::function<void(InputContext*)> callback) : CandidateWord(std::move(text)), callback_(std::move(callback)) {}

    void AppModeCandidateWord::select(InputContext* ic) const {
        if (callback_) {
            callback_(ic);
        }
    }

} // namespace fcitx
