/*
 * SPDX-FileCopyrightText: 2022-2022 CSSlayer <wengxt@gmail.com>
 * SPDX-FileCopyrightText: 2025 Võ Ngô Hoàng Thành <thanhpy2009@gmail.com>
 * SPDX-FileCopyrightText: 2026 Nguyễn Hoàng Kỳ  <nhktmdzhg@gmail.com>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/**
 * @file lilypad-config.h
 * @brief Configuration definitions for fcitx5-lilypad input method.
 */

#ifndef _FCITX5_LILYPAD_CONFIG_H_
#define _FCITX5_LILYPAD_CONFIG_H_

#include <cstdint>
#include <fcitx-config/configuration.h>
#include <fcitx-config/enum.h>
#include <fcitx-utils/i18n.h>
#include <fcitx-utils/stringutils.h>

namespace fcitx {

    /**
     * @brief Operating modes for the Lilypad input method.
     */
    enum class LilypadMode : std::uint8_t {
        Off,
        Smooth,
        SuperSmooth,
        Uinput,
        SurroundingText,
        Preedit,
        Emoji,
        Minecraft,
        Sequence,
    };

    FCITX_CONFIG_ENUM_NAME_WITH_I18N(LilypadMode, N_("OFF"), N_("Uinput (Smooth)"), N_("Uinput (Super Smooth)"), N_("Uinput (Slow)"), N_("Surrounding Text"), N_("Preedit"),
                                     N_("Emoji Picker"), N_("Minecraft"), N_("Sequence"));

    /**
     * @brief Converts LilypadMode to int and vice versa.
     */
    int       modeToInt(LilypadMode mode);
    LilypadMode intToMode(int mode);

    /**
     * @brief W2U mode for w to ư conversion.
     */
    enum class W2UMode : std::uint8_t {
        Disabled   = 0,
        NonStart   = 1,
        Everywhere = 2,
    };

    FCITX_CONFIG_ENUM_NAME_WITH_I18N(W2UMode, N_("Disabled"), N_("Non-Start"), N_("Everywhere"));

    /**
     * @brief Bracket transform mode for [ -> ơ, ] -> ư conversion.
     */
    enum class BracketTransformMode : std::uint8_t {
        Disabled   = 0,
        NonStart   = 1,
        Everywhere = 2,
    };

    FCITX_CONFIG_ENUM_NAME_WITH_I18N(BracketTransformMode, N_("Disabled"), N_("Non-Start"), N_("Everywhere"));

    /**
     * @brief Icon theme options.
     */
    enum class IconTheme : std::uint8_t {
        Auto,
        Light,
        Dark,
    };

    FCITX_CONFIG_ENUM_NAME_WITH_I18N(IconTheme, N_("Auto"), N_("Light"), N_("Dark"));

    struct InputMethodConstrain;
    struct InputMethodAnnotation;

    using InputMethodOption = Option<std::string, InputMethodConstrain, DefaultMarshaller<std::string>, InputMethodAnnotation>;

    /**
     * @brief Annotation for string list options in configuration UI.
     */
    struct StringListAnnotation : public EnumAnnotation {
        /**
         * @brief Sets the string list.
         * @param list Vector of strings to set.
         */
        void setList(std::vector<std::string> list) {
            list_ = std::move(list);
        }

        /**
         * @brief Gets the string list.
         * @return Reference to the list.
         */
        const auto& list() {
            return list_;
        }

        /**
         * @brief Dumps description to config.
         * @param config Config to write to.
         */
        void dumpDescription(RawConfig& config) const {
            EnumAnnotation::dumpDescription(config);
            config.setValueByPath("IsEnum", "True");
            for (size_t i = 0; i < list_.size(); ++i) {
                config.setValueByPath("Enum/" + std::to_string(i), list_[i]);
            }
        }

      protected:
        std::vector<std::string> list_; // NOLINT
    };

    struct InputMethodAnnotation : public StringListAnnotation {
        /**
         * @brief Dumps description with sub-config paths.
         * @param config Config to write to.
         */
        void dumpDescription(RawConfig& config) const {
            StringListAnnotation::dumpDescription(config);
            config.setValueByPath("LaunchSubConfig", "True");
        }
    };

    /**
     * @brief Annotation for time format list.
     */
    struct TimeFormatAnnotation : public StringListAnnotation {
        TimeFormatAnnotation() {
            list_ = {"%H:%M", "%H:%M:%S", "%I:%M:%S %p", "%I:%M %p", ""};
        }
    };

    /**
     * @brief Annotation for date format list.
     */
    struct DateFormatAnnotation : public StringListAnnotation {
        DateFormatAnnotation() {
            list_ = {"%d/%m/%Y", "%m/%d/%Y", "%Y-%m-%d", "%d/%m/%y", "%y-%m-%d", ""};
        }
    };

    /**
     * @brief Constraint validator for input method options.
     */
    struct InputMethodConstrain {
        using Type = std::string;

        /**
         * @brief Constructs with option pointer.
         * @param option Pointer to input method option.
         */
        InputMethodConstrain(const InputMethodOption* option) : option_(option) {}

        /**
         * @brief Validates if name is in the allowed list.
         * @param name Name to check.
         * @return True if valid.
         */
        bool check(const std::string& name) const {
            const auto& list = option_->annotation().list();
            if (list.empty()) {
                return true;
            }
            return std::find(list.begin(), list.end(), name) != list.end();
        }

        /**
         * @brief Dumps description (no-op).
         * @param config Unused.
         */
        void dumpDescription(RawConfig& /*unused*/) const {}

      private:
        const InputMethodOption* option_;
    };

    FCITX_CONFIGURATION(lilypadKeymap, Option<std::string> key{this, "Key", _("Key"), ""}; Option<std::string> value{this, "Value", _("Value"), ""};);

    FCITX_CONFIGURATION(lilypadMacroTable,
                        OptionWithAnnotation<std::vector<lilypadKeymap>, ListDisplayOptionAnnotation> macros{
                            this, "Macro", _("Macro"), {}, {}, {}, ListDisplayOptionAnnotation("Key")};);

    FCITX_CONFIGURATION(lilypadCustomKeymap,
                        OptionWithAnnotation<std::vector<lilypadKeymap>, ListDisplayOptionAnnotation> customKeymap{
                            this, "CustomKeymap", _("Custom Keymap"), {}, {}, {}, ListDisplayOptionAnnotation("Key")};);

    FCITX_CONFIGURATION(lilypadAppRule, Option<std::string> app{this, "App", _("App"), ""}; Option<int> mode{this, "Mode", _("Mode"), 0};);
    FCITX_CONFIGURATION(lilypadAppRules,
                        OptionWithAnnotation<std::vector<lilypadAppRule>, ListDisplayOptionAnnotation> rules{
                            this, "Rules", _("Rules"), {}, {}, {}, ListDisplayOptionAnnotation("App")};);

    /**
     * @brief Main configuration structure for Lilypad input method.
     */
    FCITX_CONFIGURATION(
        lilypadConfig,

        OptionWithAnnotation<LilypadMode, LilypadModeI18NAnnotation>                                         mode{this, "Mode", _("Mode"), LilypadMode::Preedit};
        Option<std::string, InputMethodConstrain, DefaultMarshaller<std::string>, InputMethodAnnotation> inputMethod{
            this, "InputMethod", _("Input Method"), "Telex", InputMethodConstrain(&inputMethod), {}, InputMethodAnnotation()};
        OptionWithAnnotation<std::string, StringListAnnotation> outputCharset{this, "OutputCharset", _("Output Charset"), "Unicode", {}, {}, StringListAnnotation()};
        KeyListOption                                           modeMenuKey{
            this, "ModeMenuKey", _("Mode Menu Hotkey"), {Key("grave")}, KeyListConstrain({KeyConstrainFlag::AllowModifierLess, KeyConstrainFlag::AllowModifierOnly})};
        KeyListOption cycleModeKey{this, "CycleModeKey", _("Cycle Mode Hotkey"), {}, KeyListConstrain({KeyConstrainFlag::AllowModifierLess, KeyConstrainFlag::AllowModifierOnly})};
        SubConfigOption                                                                appRules{this, "AppRules", _("App Rules"), "fcitx://config/addon/lilypad/app_rules"};
        OptionWithAnnotation<W2UMode, W2UModeI18NAnnotation>                           w2u{this, "W2U", _("Type w to Produce ư"), W2UMode::NonStart};
        OptionWithAnnotation<BracketTransformMode, BracketTransformModeI18NAnnotation> bracketTransform{this, "BracketTransform", _("Type [ -> ơ, ] -> ư, { -> Ơ, } -> Ư"),
                                                                                                        BracketTransformMode::Disabled};

        Option<bool> spellCheck{this, "SpellCheck", _("Enable Spell Check"), true}; Option<bool> enableMacro{this, "EnableMacro", _("Enable Macro"), true};
        Option<bool>                                                                             capitalizeMacro{this, "CapitalizeMacro", _("Capitalize Macro"), true};
        Option<bool>        autoCapitalizeAfterPunctuation{this, "AutoCapitalizeAfterPunctuation", _("Auto capitalize after sentence-ending punctuation (. ! ? Enter)"), false};
        Option<bool>        doubleSpaceToPeriod{this, "DoubleSpaceToPeriod", _("Double Space to Period"), false};
        Option<bool>        doubleHyphenToEmDash{this, "DoubleHyphenToEmDash", _("Double Hyphen to Em-Dash (--)"), false};
        Option<bool>        autoNonVnRestore{this, "AutoNonVnRestore", _("Auto Restore Invalid Words"), true};
        Option<bool>        modernStyle{this, "ModernStyle", _("Use oà, uý (Instead Of òa, úy)"), true};
        Option<bool>        freeMarking{this, "FreeMarking", _("Allow Type With More Freedom"), true};
        Option<bool>        ddFreeStyle{this, "DdFreeStyle", _("Allow dd To Produce đ When Auto Restore Invalid Words Is On"), true};
        Option<bool>        fixUinputWithAck{this, "FixUinputWithAck", _("Fix Uinput Mode With Ack"), false};
        Option<bool>        useLilypadIcons{this, "UseLilypadIcons", _("Use Lilypad Status Icons"), true};
        Option<bool>        enableIkiAdaptive{this, "EnableIkiAdaptive", _("Enable Adaptive Inter-Keystroke Interval (IKI) Engine"), true};
        Option<int>         ikiMinMs{this, "IkiMinMs", _("Minimum IKI threshold (ms)"), 10};
        Option<int>         ikiMaxMs{this, "IkiMaxMs", _("Maximum IKI threshold (ms)"), 500};

        Option<bool>        enableDictionary{this, "EnableDictionary", _("Custom Dictionary"), false};
        Option<bool>        enableCustomKeymap{this, "EnableCustomKeymap", _("Custom Keymap"), false};

        Option<bool>        showModeSmooth{this, "ShowModeSmooth", _("Show Uinput (Smooth)"), true};
        Option<std::string> shortcutSmooth{this, "ShortcutSmooth", _("Shortcut for Uinput (Smooth)"), "1"};
        Option<bool>        showModeUinput{this, "ShowModeUinput", _("Show Uinput (Slow)"), true};
        Option<std::string> shortcutUinput{this, "ShortcutUinput", _("Shortcut for Uinput (Slow)"), "2"};
        Option<bool>        showModeSuperSmooth{this, "ShowModeSuperSmooth", _("Show Uinput (Super Smooth)"), true};
        Option<std::string> shortcutSuperSmooth{this, "ShortcutSuperSmooth", _("Shortcut for Uinput (Super Smooth)"), "a"};
        Option<bool>        showModeMinecraft{this, "ShowModeMinecraft", _("Show Minecraft"), true};
        Option<std::string> shortcutMinecraft{this, "ShortcutMinecraft", _("Shortcut for Minecraft"), "3"};
        Option<bool>        showModeSequence{this, "ShowModeSequence", _("Show Sequence"), true};
        Option<std::string> shortcutSequence{this, "ShortcutSequence", _("Shortcut for Sequence"), "s"};
        Option<bool>        showModeSurroundingText{this, "ShowModeSurroundingText", _("Show Surrounding Text"), true};
        Option<std::string> shortcutSurroundingText{this, "ShortcutSurroundingText", _("Shortcut for Surrounding Text"), "4"};
        Option<bool>        showModePreedit{this, "ShowModePreedit", _("Show Preedit"), true};
        Option<std::string> shortcutPreedit{this, "ShortcutPreedit", _("Shortcut for Preedit"), "q"};
        Option<bool>        showModeEmoji{this, "ShowModeEmoji", _("Show Emoji Picker"), true};
        Option<std::string> shortcutEmoji{this, "ShortcutEmoji", _("Shortcut for Emoji Picker"), "w"}; Option<bool> showModeOff{this, "ShowModeOff", _("Show OFF"), true};
        Option<std::string> shortcutOff{this, "ShortcutOff", _("Shortcut for OFF"), "e"}; Option<bool> showModeDefault{this, "ShowModeDefault", _("Show Default Typing"), true};
        Option<std::string> shortcutDefault{this, "ShortcutDefault", _("Shortcut for Default Typing"), "r"};
        Option<bool>        enableMacroInOffMode{this, "EnableMacroInOffMode", _("Allow Macro in Off Mode"), false};

        Option<std::string> modeOrder{this, "ModeOrder", _("Mode Order"), "Smooth,Uinput,Minecraft,SurroundingText,Preedit,Emoji,Sequence,Off,SuperSmooth,Default"};

        OptionWithAnnotation<std::string, TimeFormatAnnotation>  timeFormat{this, "TimeFormat", _("Time Format ($TIME in macro)"), "%H:%M", {}, {}, TimeFormatAnnotation()};
        OptionWithAnnotation<std::string, DateFormatAnnotation>  dateFormat{this, "DateFormat", _("Date Format ($DATE in macro)"), "%d/%m/%Y", {}, {}, DateFormatAnnotation()};

        SubConfigOption                                          macroEditor{this, "MacroEditor", _("Macro"), "fcitx://config/addon/lilypad/lilypad-macro"};
        SubConfigOption                                          customKeymap{this, "CustomKeymap", _("Custom Keymap"), "fcitx://config/addon/lilypad/custom_keymap"};
        OptionWithAnnotation<IconTheme, IconThemeI18NAnnotation> iconTheme{this, "IconTheme", _("Icon Color"), IconTheme::Auto};);

} // namespace fcitx

#endif
