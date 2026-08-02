/*
 * SPDX-FileCopyrightText: 2026 Nguyễn Hoàng Kỳ  <nhktmdzhg@gmail.com>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/**
 * @file emoji-entry.h
 * @brief Emoji entry data structure.
 */

#ifndef EMOJI_ENTRY_H
#define EMOJI_ENTRY_H

#include <string>

struct EmojiEntry {
    std::string trigger;
    std::string output;
};
#endif // EMOJI_ENTRY_H