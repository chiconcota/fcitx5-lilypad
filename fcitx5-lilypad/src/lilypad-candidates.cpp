/*
 * SPDX-FileCopyrightText: 2022-2022 CSSlayer <wengxt@gmail.com>
 * SPDX-FileCopyrightText: 2025-2026 Lý Tất Thành
 * SPDX-FileCopyrightText: 2026 Antigravity
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

#include "lilypad-candidates.h"
#include "lilypad-state.h"
#include "lilypad-engine.h"
#include "emoji.h"

#include <fcitx-utils/i18n.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputpanel.h>
#include <fcitx/userinterface.h>

#include <algorithm>
#include <memory>
#include <string>
#include <utility>

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

    void LilypadState::updateEmojiPageStatus(CommonCandidateList* commonList) {
        if ((commonList == nullptr) || commonList->empty()) {
            return;
        }

        int pageSize = commonList->pageSize();
        if (pageSize <= 0) {
            pageSize = 9;
        }

        int         totalItems  = commonList->totalSize();
        int         currentPage = commonList->currentPage() + 1;
        int         totalPages  = (totalItems + pageSize - 1) / pageSize;

        std::string status = _("Page ") + std::to_string(currentPage) + "/" + std::to_string(totalPages);
        ic_->inputPanel().setAuxDown(Text(status));
    }

    void LilypadState::handleEmojiMode(KeyEvent& keyEvent) {
        const KeySym currentSym      = keyEvent.rawKey().sym();
        bool         isCtrlBackspace = isBackspace(currentSym) && ((keyEvent.rawKey().states() & KeyState::Ctrl) != 0U);

        if (keyEvent.key().hasModifier() && !isCtrlBackspace) {
            keyEvent.forward();
            return;
        }

        auto baseList   = ic_->inputPanel().candidateList();
        auto commonList = std::dynamic_pointer_cast<CommonCandidateList>(baseList);
        if (commonList && currentSym >= FcitxKey_1 && currentSym <= FcitxKey_9) {
            int offset      = currentSym - FcitxKey_1;
            int globalIndex = (commonList->currentPage() * commonList->pageSize()) + offset;

            if (globalIndex < commonList->totalSize()) {
                commonList->candidateFromAll(globalIndex).select(ic_);
                keyEvent.filterAndAccept();
                return;
            }
        }

        if (commonList && !commonList->empty()) {
            int  globalCursorIndex = commonList->globalCursorIndex();
            int  totalSize         = commonList->totalSize();
            int  currentPage       = commonList->currentPage();
            int  pageSize          = commonList->pageSize();
            int  localCursorIndex  = globalCursorIndex - (currentPage * pageSize);

            bool handled = false;

            switch (currentSym) {
                case FcitxKey_Tab:
                case FcitxKey_Down: {
                    if (localCursorIndex < pageSize - 1 && globalCursorIndex < totalSize - 1) {
                        commonList->setGlobalCursorIndex(globalCursorIndex + 1);
                    } else {
                        commonList->setGlobalCursorIndex(currentPage * pageSize);
                    }
                    handled = true;
                    break;
                }

                case FcitxKey_ISO_Left_Tab:
                case FcitxKey_Up: {
                    if (localCursorIndex > 0) {
                        commonList->setGlobalCursorIndex(globalCursorIndex - 1);
                    } else {
                        int lastIndex = std::min((currentPage * pageSize) + pageSize - 1, totalSize - 1);
                        commonList->setGlobalCursorIndex(lastIndex);
                    }
                    handled = true;
                    break;
                }
                case FcitxKey_Page_Down:
                case FcitxKey_Right: {
                    if (commonList->hasNext()) {
                        commonList->next();
                        int newPage = commonList->currentPage();
                        commonList->setGlobalCursorIndex(newPage * pageSize);
                        handled = true;
                    }
                    break;
                }
                case FcitxKey_Page_Up:
                case FcitxKey_Left: {
                    if (commonList->hasPrev()) {
                        commonList->prev();
                        int newPage = commonList->currentPage();
                        commonList->setGlobalCursorIndex(newPage * pageSize);
                        handled = true;
                    }
                    break;
                }
                default: break;
            }

            if (handled) {
                updateEmojiPageStatus(commonList.get());
                ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
                keyEvent.filterAndAccept();
                return;
            }
        }

        if (isBackspace(currentSym)) {
            if (!emojiBuffer_.empty()) {
                if (isCtrlBackspace) {
                    emojiBuffer_.clear();
                } else {
                    emojiBuffer_.pop_back();
                    while (!emojiBuffer_.empty() && (emojiBuffer_.back() & 0xC0) == 0x80) {
                        emojiBuffer_.pop_back();
                    }
                }
                keyEvent.filterAndAccept();
            } else {
                keyEvent.forward();
            }
            updateEmojiPreedit();
            return;
        }

        switch (currentSym) {
            case FcitxKey_space:
            case FcitxKey_Return: {
                if (commonList && !commonList->empty()) {
                    int globalIdx = commonList->globalCursorIndex();
                    commonList->candidateFromAll(globalIdx).select(ic_);
                    keyEvent.filterAndAccept();
                } else if (currentSym == FcitxKey_Return && !emojiBuffer_.empty()) {
                    ic_->commitString(emojiBuffer_);
                    emojiBuffer_.clear();
                    updateEmojiPreedit();
                    keyEvent.filterAndAccept();
                } else {
                    keyEvent.forward();
                }
                return;
            }

            case FcitxKey_Escape: {
                emojiBuffer_.clear();
                emojiCandidates_.clear();
                ic_->inputPanel().reset();
                ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
                keyEvent.filterAndAccept();
                return;
            }

            default: break;
        }

        {
            std::string utf8Char = Key::keySymToUTF8(currentSym);
            if (!utf8Char.empty()) {
                emojiBuffer_.append(utf8Char);
                keyEvent.filterAndAccept();
                updateEmojiPreedit();
            } else {
                keyEvent.forward();
            }
        }
    }

    void LilypadState::updateEmojiPreedit() {
        if (emojiBuffer_.empty()) {
            emojiCandidates_ = engine_->emojiLoader().history();
            if (emojiCandidates_.empty()) {
                ic_->inputPanel().reset();
                ic_->updatePreedit();
                ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
                return;
            }
        } else {
            emojiCandidates_ = engine_->emojiLoader().search(emojiBuffer_);
        }

        if (!emojiBuffer_.empty()) {
            Text preeditText;
            preeditText.append(emojiBuffer_, TextFormatFlag::Underline);
            preeditText.setCursor(static_cast<int>(preeditText.textLength()));
            if (ic_->capabilityFlags().test(CapabilityFlag::Preedit))
                ic_->inputPanel().setClientPreedit(preeditText);
            else
                ic_->inputPanel().setPreedit(preeditText);
        } else {
            ic_->inputPanel().setClientPreedit(Text());
            ic_->inputPanel().setPreedit(Text());
        }

        if (!emojiCandidates_.empty()) {
            auto candidateList = std::make_unique<CommonCandidateList>();
            candidateList->setLayoutHint(CandidateLayoutHint::Vertical);
            candidateList->setPageSize(9);

            for (size_t i = 0; i < emojiCandidates_.size(); ++i) {
                size_t localIndex = (i % 9) + 1;
                Text   displayLabel;
                if (emojiBuffer_.empty()) {
                    displayLabel.append(std::to_string(localIndex) + ": " + emojiCandidates_[i].output, TextFormatFlag::NoFlag);
                } else {
                    displayLabel.append(std::to_string(localIndex) + ": " + emojiCandidates_[i].trigger + " " + emojiCandidates_[i].output, TextFormatFlag::NoFlag);
                }
                candidateList->append(std::make_unique<EmojiCandidateWord>(displayLabel, this, emojiCandidates_[i]));
            }
            candidateList->setGlobalCursorIndex(0);

            ic_->inputPanel().setCandidateList(std::move(candidateList));
            auto currentList = std::dynamic_pointer_cast<CommonCandidateList>(ic_->inputPanel().candidateList());
            updateEmojiPageStatus(currentList.get());
        } else {
            ic_->inputPanel().setCandidateList(nullptr);
        }

        ic_->updatePreedit();
        ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
    }

} // namespace fcitx
