/*
 * SPDX-FileCopyrightText: 2026 Nguyễn Hoàng Kỳ  <nhktmdzhg@gmail.com>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/**
 * @file lotus-candidates.h
 * @brief Candidate word classes for Lotus input method UI.
 */

#ifndef _FCITX5_LOTUS_CANDIDATES_H_
#define _FCITX5_LOTUS_CANDIDATES_H_

#include <fcitx/candidatelist.h>
#include <functional>
#include "emoji-entry.h"

namespace fcitx {

    class LotusState;

    /**
     * @brief Candidate word for emoji selection.
     *
     * Displays emoji candidates and commits selected emoji to input.
     */
    class EmojiCandidateWord : public CandidateWord {
      public:
        /**
         * @brief Constructs an emoji candidate.
         * @param text Display text for the candidate.
         * @param state Pointer to LotusState.
         * @param entry The emoji entry.
         */
        EmojiCandidateWord(Text text, LotusState* state, EmojiEntry entry);

        /**
         * @brief Handles candidate selection.
         * @param inputContext Current input context.
         */
        void select(InputContext* inputContext) const override;

      private:
        LotusState* state_;
        EmojiEntry  entry_;
    };

    /**
     * @brief Candidate word for application mode selection.
     *
     * Used in the application-specific input mode menu.
     */
    class AppModeCandidateWord : public CandidateWord {
      public:
        /**
         * @brief Constructs an app mode candidate.
         * @param text Display text for the candidate.
         * @param callback Function to call when selected.
         */
        AppModeCandidateWord(Text text, std::function<void(InputContext*)> callback);

        /**
         * @brief Handles candidate selection.
         * @param ic Current input context.
         */
        void select(InputContext* ic) const override;

      private:
        std::function<void(InputContext*)> callback_;
    };

} // namespace fcitx

#endif // _FCITX5_LOTUS_CANDIDATES_H_
