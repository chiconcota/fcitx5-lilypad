/*
 * SPDX-FileCopyrightText: 2022-2022 CSSlayer <wengxt@gmail.com>
 * SPDX-FileCopyrightText: 2025 Võ Ngô Hoàng Thành <thanhpy2009@gmail.com>
 * SPDX-FileCopyrightText: 2026 Nguyễn Hoàng Kỳ  <nhktmdzhg@gmail.com>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */
#include "lilypad-engine.h"
#include "fcitx-utils/keysym.h"
#include "lilypad-config.h"
#include "lilypad-state.h"
#include "lilypad-candidates.h"
#include "lilypad-monitor.h"
#include "lilypad-utils.h"
#include "ack-apps.h"
#include <optional>
#include <sys/socket.h>
#include <utility>

#include <fcitx-config/iniparser.h>
#include <fcitx/menu.h>
#include <fcitx/userinterfacemanager.h>
#include <fcitx/inputmethodmanager.h>
#include <fcitx-utils/event.h>
#include <fcitx-utils/utf8.h>
#include <fcitx-utils/eventdispatcher.h>

#include <atomic>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <unordered_set>

#include <fcntl.h>
#include <sstream>

namespace fcitx {
    constexpr const char* CharsetActionPrefix = "lilypad-charset-";
    const std::string     CustomKeymapFile    = "conf/lilypad-custom-keymap.conf";
    const std::string     MacroTableFile      = "conf/lilypad-macro-table.conf";

    int                   modeToInt(LilypadMode mode) {
        switch (mode) {
            case LilypadMode::Off: return 0;
            case LilypadMode::Smooth: return 1;
            case LilypadMode::Uinput: return 2;
            case LilypadMode::SuperSmooth: return 3;
            case LilypadMode::SurroundingText: return 4;
            case LilypadMode::Preedit: return 5;
            case LilypadMode::Emoji: return 6;
            case LilypadMode::Minecraft: return 8;
            case LilypadMode::Sequence: return 9;
            default: return 0;
        }
    }

    LilypadMode intToMode(int mode) {
        switch (mode) {
            case 0: return LilypadMode::Off;
            case 1: return LilypadMode::Smooth;
            case 2: return LilypadMode::Uinput;
            case 3: return LilypadMode::SuperSmooth;
            case 4: return LilypadMode::SurroundingText;
            case 5: return LilypadMode::Preedit;
            case 6: return LilypadMode::Emoji;
            case 8: return LilypadMode::Minecraft;
            case 9: return LilypadMode::Sequence;
            default: return LilypadMode::Off;
        }
    }

    // Returns the KeySym that triggers the "Type hotkey char" action in the mode
    // menu.  If the hotkey itself conflicts with a reserved menu key, falls back
    // to FcitxKey_f.
    static bool isAppModeMenuReservedKey(KeySym sym, const lilypadConfig& config) {
        if (sym == Key(*config.shortcutSmooth).sym() || sym == Key(*config.shortcutUinput).sym() || sym == Key(*config.shortcutMinecraft).sym() ||
            sym == Key(*config.shortcutSurroundingText).sym() || sym == Key(*config.shortcutPreedit).sym() || sym == Key(*config.shortcutEmoji).sym() ||
            sym == Key(*config.shortcutOff).sym() || sym == Key(*config.shortcutSuperSmooth).sym() || sym == Key(*config.shortcutDefault).sym()) {
            return true;
        }

        switch (sym) {
            case FcitxKey_Escape:
            case FcitxKey_Tab:
            case FcitxKey_ISO_Left_Tab:
            case FcitxKey_Return:
            case FcitxKey_space:
            case FcitxKey_Up:
            case FcitxKey_Down: return true;
            default: return false;
        }
    }

    static KeySym typeKeyForModeMenuHotkey(KeySym hotkeySym, const lilypadConfig& config) {
        return isAppModeMenuReservedKey(hotkeySym, config) ? FcitxKey_f : hotkeySym;
    }

    bool LilypadEngine::isDarkMode() {
        FILE* pipe = popen("dbus-send --session --dest=org.freedesktop.portal.Desktop --print-reply /org/freedesktop/portal/desktop org.freedesktop.portal.Settings.ReadOne "
                           "string:'org.freedesktop.appearance' string:'color-scheme' 2>/dev/null",
                           "r");
        if (pipe != nullptr) {
            char buffer[256];
            while (fgets(buffer, sizeof(buffer), pipe) != nullptr) {
                uint32_t value = 0;
                if (sscanf(buffer, "%*[^v]variant uint32 %u", &value) == 1) {
                    pclose(pipe);
                    return value == 1;
                }
            }
            pclose(pipe);
        }

        pipe = popen("gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null", "r");
        if (pipe != nullptr) {
            char buffer[256];
            if (fgets(buffer, sizeof(buffer), pipe) != nullptr) {
                pclose(pipe);
                return strstr(buffer, "prefer-dark") != nullptr;
            }
            pclose(pipe);
        }

        return false;
    }

    static inline uintptr_t newMacroTable(const lilypadMacroTable& macroTable) {
        const auto&        macros = *macroTable.macros;
        std::vector<char*> charArray;
        charArray.reserve((macros.size() * 2) + 1);
        for (const auto& keymap : macros) {
            // External C API doesn't use const, but doesn't modify data
            charArray.push_back(const_cast<char*>(keymap.key->data()));   //NOLINT
            charArray.push_back(const_cast<char*>(keymap.value->data())); //NOLINT
        }
        charArray.push_back(nullptr);
        return NewMacroTable(charArray.data());
    }

    static inline std::vector<std::string> convertToStringList(char** list) {
        std::vector<std::string> result;
        if (list != nullptr) {
            for (size_t i = 0; list[i] != nullptr; ++i) { //NOLINT
                result.emplace_back(list[i]);             //NOLINT
                free(list[i]);                            //NOLINT
            }
            free(list); //NOLINT
        }
        return result;
    }

    uintptr_t LilypadEngine::macroTable() const {
        if (config_.inputMethod.value().empty()) {
            return 0;
        }
        return macroTableObject_.handle();
    }

    LilypadEngine::LilypadEngine(Instance* instance) : instance_(instance), factory_([this](InputContext& ic) { return new LilypadState(this, &ic); }) { //NOLINT
        const char* desktop = std::getenv("XDG_CURRENT_DESKTOP");
        isGnome_            = (desktop != nullptr) && std::string(desktop).find("GNOME") != std::string::npos;
        // emptyCustomKeymap_.customKeymap is implicitly initialized to empty by fcitx::Option default value macro.
        Init();
        {
            auto imNames = convertToStringList(GetInputMethodNames());
            imNames.push_back("Custom");
            imNames_ = std::move(imNames);
        }
        config_.inputMethod.annotation().setList(imNames_);

        auto& uiManager = instance_->userInterfaceManager();

        charsetAction_ = std::make_unique<SimpleAction>();
        charsetAction_->setShortText(_("Charset"));
        charsetAction_->setIcon("character-set");
        uiManager.registerAction("lilypad-charset", charsetAction_.get());
        charsetMenu_ = std::make_unique<Menu>();
        charsetAction_->setMenu(charsetMenu_.get());

        auto charsets = convertToStringList(GetCharsetNames());
        for (const auto& charset : charsets) {
            charsetSubAction_.emplace_back(std::make_unique<SimpleAction>());
            auto* action = charsetSubAction_.back().get();
            action->setShortText(charset);
            action->setCheckable(true);
            uiManager.registerAction(stringutils::concat(CharsetActionPrefix, charset), action);
            connections_.emplace_back(action->connect<SimpleAction::Activated>([this, charset](InputContext* ic) {
                if (config_.outputCharset.value() == charset)
                    return;
                config_.outputCharset.setValue(charset);
                saveConfig();
                refreshEngine();
                updateCharsetAction(ic);
                if (ic)
                    ic->updateUserInterface(UserInterfaceComponent::StatusArea);
            }));
            charsetMenu_->addAction(action);
        }
        config_.outputCharset.annotation().setList(charsets);

        initToggleAction(spellCheckAction_, config_.spellCheck, "lilypad-spellcheck", "tools-check-spelling", _("Spell Check"), _("Spell Check"), uiManager);
        initToggleAction(macroAction_, config_.enableMacro, "lilypad-macro", "document-edit", _("Macro"), _("Macro"), uiManager);
        initToggleAction(capitalizeMacroAction_, config_.capitalizeMacro, "lilypad-capitalizemacro", "format-text-uppercase", _("Capitalize Macro"), _("Capitalize Macro"),
                         uiManager);
        initToggleAction(autoNonVnRestoreAction_, config_.autoNonVnRestore, "lilypad-autonvnrestore", "edit-undo", _("Auto Restore Invalid Words"), _("Auto Non-VN Restore"),
                         uiManager);
        initToggleAction(enableDictionaryAction_, config_.enableDictionary, "lilypad-dictionary", "accessories-dictionary", _("Custom Dictionary"), _("Custom Dictionary"),
                         uiManager);

        settingsAction_ = std::make_unique<SimpleAction>();
        settingsAction_->setShortText(_("Settings"));
        settingsAction_->setIcon("configure");
        connections_.emplace_back(settingsAction_->connect<SimpleAction::Activated>([](InputContext*) {
            if (fork() == 0) {
                execl(FCITX5_LILYPAD_SETTINGS_PATH, FCITX5_LILYPAD_SETTINGS_PATH, nullptr);
                _exit(1);
            }
        }));
        uiManager.registerAction("lilypad-settings", settingsAction_.get());

#if LILYPAD_USE_MODERN_FCITX_API
        std::string configDir = (StandardPaths::global().userDirectory(StandardPathsType::Config) / "fcitx5" / "conf").string();
#else
        std::string configDir = StandardPath::global().userDirectory(StandardPath::Type::Config) + "/fcitx5/conf";
#endif

        if (!std::filesystem::exists(configDir)) {
            std::filesystem::create_directories(configDir);
        }
        reloadConfig();
        instance_->inputContextManager().registerProperty("LilypadState", &factory_);
        appRulesPath_ = configDir + "/lilypad-app-rules.conf";
        loadAppRules();
        toggleActions_ = {charsetAction_.get(),          spellCheckAction_.get(),       macroAction_.get(),   capitalizeMacroAction_.get(),
                          autoNonVnRestoreAction_.get(), enableDictionaryAction_.get(), settingsAction_.get()};
    }

    void LilypadEngine::initToggleAction(std::unique_ptr<SimpleAction>& action, Option<bool>& option, const std::string& actionId, const std::string& iconName,
                                         const std::string& textLong, const std::string& textOnOff, UserInterfaceManager& uiManager) {
        action = std::make_unique<SimpleAction>();
        action->setShortText(textLong);
        action->setIcon(iconName);
        action->setCheckable(false);
        connections_.emplace_back(action->connect<SimpleAction::Activated>([this, &action, &option, textOnOff](InputContext* ic) {
            option.setValue(!option.value());
            saveConfig();
            refreshOption();
            updateAction(ic, action, option, textOnOff);
        }));
        uiManager.registerAction(actionId, action.get());
    }

    void LilypadEngine::updateAction(InputContext* ic, std::unique_ptr<SimpleAction>& action, Option<bool>& option, const std::string& textOnOff) {
        action->setShortText((option.value() ? "✔ " : "✖ ") + textOnOff);
        if (ic != nullptr) {
            action->update(ic);
        }
    }

    LilypadEngine::~LilypadEngine() {
        stop_flag_monitor.store(true, std::memory_order_release);
        int fd = mouse_socket_fd.load(std::memory_order_acquire);
        if (fd >= 0) {
            shutdown(fd, SHUT_RDWR);
        }
        if (mouse_thread.joinable()) {
            mouse_thread.join();
        }
        int old_fd = uinput_client_fd_.exchange(-1);
        if (old_fd != -1) {
            close(old_fd);
        }
        LILYPAD_INFO("Engine destroyed.");
    }

    const lilypadCustomKeymap& LilypadEngine::customKeymap() const {
        if (config_.enableCustomKeymap.value()) {
            return customKeymap_;
        }
        return emptyCustomKeymap_;
    }

    void LilypadEngine::reloadConfig() {
        readAsIni(config_, "conf/lilypad.conf");
        readAsIni(customKeymap_, CustomKeymapFile);
        readAsIni(macroTables_, MacroTableFile);
        macroTableObject_.reset(newMacroTable(macroTables_));
        if (config_.enableDictionary.value()) {
#if LILYPAD_USE_MODERN_FCITX_API
            auto fd = StandardPaths::global().open(StandardPathsType::PkgData, "lilypad/vietnamese.cm.dict");
#else
            auto fd = StandardPath::global().open(StandardPath::Type::PkgData, "lilypad/vietnamese.cm.dict", O_RDONLY);
#endif
            if (fd.isValid()) {
                dictionary_.reset(NewDictionary(fd.release()));
            }
        } else {
#if LILYPAD_USE_MODERN_FCITX_API
            auto paths = StandardPaths::global().locateAll(StandardPathsType::PkgData, "lilypad/vietnamese.cm.dict");
#else
            auto paths = StandardPath::global().locateAll(StandardPath::Type::PkgData, "lilypad/vietnamese.cm.dict");
#endif
            for (const auto& p : paths) {
#if LILYPAD_USE_MODERN_FCITX_API
                if (!isStartsWith(p.string(), "/home/")) {
                    auto fd = fcitx::UnixFD(::open(p.c_str(), O_RDONLY));
                    if (fd.isValid()) {
                        dictionary_.reset(NewDictionary(fd.release()));
#else
                if (!isStartsWith(p, "home/")) {
                    int fd = ::open(p.c_str(), O_RDONLY);
                    if (fd != -1) {
                        dictionary_.reset(NewDictionary(fd));
#endif
                        break;
                    }
                }
            }
        }
        loadAppRules();
        populateConfig();
    }

    const Configuration* LilypadEngine::getSubConfig(const std::string& path) const {
        if (path == "custom_keymap")
            return &customKeymap_;
        if (path == "lilypad-macro") {
            return &macroTables_;
        }
        if (path == "app_rules") {
            return &appRulesTables_;
        }
        return nullptr;
    }

    void LilypadEngine::setConfig(const RawConfig& config) {
        config_.load(config, true);
        saveConfig();
        populateConfig();
    }

    void LilypadEngine::populateConfig() {
        refreshEngine();
        refreshOption();
        updateCharsetAction(nullptr);
        updateAction(nullptr, spellCheckAction_, config_.spellCheck, _("Spell Check"));
        updateAction(nullptr, macroAction_, config_.enableMacro, _("Macro"));
        updateAction(nullptr, capitalizeMacroAction_, config_.capitalizeMacro, _("Capitalize Macro"));
        updateAction(nullptr, autoNonVnRestoreAction_, config_.autoNonVnRestore, _("Auto Non-VN Restore"));
        updateAction(nullptr, enableDictionaryAction_, config_.enableDictionary, _("Custom Dictionary"));
    }

    void LilypadEngine::setSubConfig(const std::string& path, const RawConfig& config) {
        if (path == "custom_keymap") {
            customKeymap_.load(config, true);
            safeSaveAsIni(customKeymap_, CustomKeymapFile);
            refreshEngine();
        } else if (path == "lilypad-macro") {
            macroTables_.load(config, true);
            safeSaveAsIni(macroTables_, MacroTableFile);
            macroTableObject_.reset(newMacroTable(macroTables_));
            refreshEngine();
        } else if (path == "app_rules") {
            appRulesTables_.load(config, true);
            {
                std::lock_guard<std::mutex> lock(appRulesMutex_);
                for (auto it = appRules_.begin(); it != appRules_.end();) {
                    if (!isStartsWith(it->first, "ctx_")) {
                        it = appRules_.erase(it);
                    } else {
                        ++it;
                    }
                }
                for (const auto& rule : *appRulesTables_.rules) {
                    appRules_[*rule.app] = intToMode(*rule.mode);
                }
            }
            saveAppRules();
            refreshEngine();
        }
    }

    std::string LilypadEngine::subMode(const InputMethodEntry& /*entry*/, InputContext& /*inputContext*/) {
        return *config_.inputMethod;
    }

    void LilypadEngine::activate(const InputMethodEntry& /*entry*/, InputContextEvent& event) {
        auto*                    ic        = event.inputContext();
        const bool               surrvalid = ic->surroundingText().isValid();
        const bool               is_dbus   = getFrontendName(ic) == "dbus";
        static std::atomic<bool> mouseThreadStarted{false};
        if (!mouseThreadStarted.exchange(true))
            startMouseReset();

        auto& statusArea = event.inputContext()->statusArea();
        if (ic->capabilityFlags().test(CapabilityFlag::Preedit))
            instance_->inputContextManager().setPreeditEnabledByDefault(true);

        std::string appName = getProgramName(ic);
        LILYPAD_INFO("App name: " + appName);

        const LilypadMode targetMode = getAppRule(appName);
        LILYPAD_INFO("Target mode: " + LilypadModeI18NAnnotation::toString(targetMode));

        updateCharsetAction(event.inputContext());

        setMode(targetMode, event.inputContext());

        auto* state = ic->propertyFor(&factory_);

        // Workaround for chromium wayland issue where suggestions cause a doubled
        // first character. Forwarding may prevent BS from being sent
        // to the client.
        //
        // Note that with chromium x11 we can't do anything to fixes this because
        // it not support surrounding text so can't know when it show suggestions
        //
        // TODO: Properly fixes instead ugly WA
        state->wa_chromium_flag = false;

        state->waitAck_ = false;
        std::string appNameLower = appName;
#if __cplusplus >= 202002L
        std::ranges::transform(appNameLower, appNameLower.begin(), ::tolower);
#else
        std::transform(appNameLower.begin(), appNameLower.end(), appNameLower.begin(), ::tolower);
#endif
        if (targetMode == LilypadMode::Uinput || targetMode == LilypadMode::Smooth || targetMode == LilypadMode::Minecraft || targetMode == LilypadMode::SuperSmooth || targetMode == LilypadMode::Sequence) {
            for (const auto& ackApp : ack_apps) {
                if (appNameLower.find(ackApp) != std::string::npos) {
                    if (is_dbus && *config_.fixUinputWithAck) {
                        state->waitAck_ = true;
                        LILYPAD_INFO(ackApp + " detected, waiting for ack");
                    }
                    state->wa_chromium_flag = true;
                    LILYPAD_INFO(ackApp + " detected: set wa_chromium_flag=true");
                    break;
                }
            }
        }
        if (event.type() == EventType::InputContextFocusIn && is_dbus && !surrvalid) {
            LILYPAD_INFO("Skip clearAllBuffers");
        } else if (surrvalid && !state->oldPreBuffer_.empty() && (now_ms() - state->lastDeactivateTime_) < 100) {
            state->clearAllBuffers();
        }
        is_deleting_.store(false);
        needEngineReset.store(false);
        if (targetMode == LilypadMode::Emoji) {
            state->updateEmojiPreedit();
        } else {
            ic->inputPanel().reset();
            ic->updateUserInterface(UserInterfaceComponent::InputPanel);
            if (realMode == LilypadMode::Preedit || realMode == LilypadMode::SurroundingText)
                ic->updatePreedit();
        }
        for (const auto& action : toggleActions_) {
            statusArea.addAction(StatusGroup::InputMethod, action);
        }
    }

    void LilypadEngine::keyEvent(const InputMethodEntry& /*entry*/, KeyEvent& keyEvent) {
        auto* ic = keyEvent.inputContext();

        if (isSelectingAppMode_ && g_mouse_clicked.load(std::memory_order_acquire)) {
            closeAppModeMenu();
            ic->inputPanel().reset();
            ic->updateUserInterface(UserInterfaceComponent::InputPanel);
            auto* state = ic->propertyFor(&factory_);
            state->commitBuffer();
            state->reset();
        }

        if (isSelectingAppMode_) {
            if (keyEvent.isRelease())
                return;

            auto   baseList = ic->inputPanel().candidateList();
            auto   menuList = std::dynamic_pointer_cast<CommonCandidateList>(baseList);
            KeySym keySym   = keyEvent.key().sym();

            auto   moveCursor = [&](int delta) {
                if (!menuList || menuList->empty()) {
                    return false;
                }

                int totalSize = menuList->totalSize();
                if (totalSize <= 1) {
                    return false;
                }

                int cursorIndex = menuList->globalCursorIndex();
                if (cursorIndex < 0 || cursorIndex >= totalSize) {
                    cursorIndex = 0;
                }

                int nextIndex = cursorIndex + delta;
                if (nextIndex < 0) {
                    nextIndex = totalSize - 1;
                } else if (nextIndex >= totalSize) {
                    nextIndex = 0;
                }

                menuList->setGlobalCursorIndex(nextIndex);
                ic->updateUserInterface(UserInterfaceComponent::InputPanel);
                return true;
            };

            keyEvent.filterAndAccept();

            std::optional<LilypadMode> selectedMode  = std::nullopt;
            bool                       selectionMade = false;

            switch (keySym) {
                case FcitxKey_Tab:
                case FcitxKey_Down: {
                    if (moveCursor(1)) {
                        return;
                    }
                    break;
                }
                case FcitxKey_ISO_Left_Tab:
                case FcitxKey_Up: {
                    if (moveCursor(-1)) {
                        return;
                    }
                    break;
                }
                case FcitxKey_space:
                case FcitxKey_Return: {
                    if (menuList && !menuList->empty()) {
                        int selectedIndex = menuList->globalCursorIndex();
                        if (selectedIndex < 0 || selectedIndex >= menuList->totalSize()) {
                            selectedIndex = 0;
                        }
                        menuList->candidateFromAll(selectedIndex).select(ic);
                        return;
                    }
                    break;
                }
                case FcitxKey_Escape: {
                    selectionMade = true;
                    break;
                }
                default: {
                    auto it = modeMenuMapping_.find(keySym);
                    if (it != modeMenuMapping_.end()) {
                        selectedMode = it->second;
                    }

                    if (selectedMode == std::nullopt) {
                        const auto& kl = *config_.modeMenuKey;
                        if (kl.size() == 1 && !kl[0].hasModifier()) {
                            std::string charStr = Key::keySymToUTF8(kl[0].sym());
                            if (!charStr.empty()) {
                                if (keySym == typeKeyForModeMenuHotkey(kl[0].sym(), config_)) {
                                    isSelectingAppMode_ = false;
                                    ic->inputPanel().reset();
                                    ic->updateUserInterface(UserInterfaceComponent::InputPanel);
                                    auto* state = ic->propertyFor(&factory_);
                                    state->commitBuffer();
                                    state->reset();
                                    ic->commitString(charStr);
                                    return;
                                }
                            }
                        }
                    }
                    break;
                }
            }

            if (selectedMode != std::nullopt) {
                LILYPAD_INFO("Selected mode: " + LilypadModeI18NAnnotation::toString(selectedMode.value()));
                if (selectedMode != LilypadMode::Emoji) {
                    if (keySym == Key(*config_.shortcutDefault).sym()) { // Default Typing key
                        std::lock_guard<std::mutex> lock(appRulesMutex_);
                        appRules_.erase(currentConfigureApp_);
                        // Remove from the configuration object too
                        auto rules = *appRulesTables_.rules;
                        rules.erase(std::remove_if(rules.begin(), rules.end(), [this](const auto& rule) { return *rule.app == currentConfigureApp_; }), rules.end());
                        appRulesTables_.rules.setValue(std::move(rules));
                    } else {
                        setAppRule(currentConfigureApp_, selectedMode.value());
                    }
                    if (!isStartsWith(currentConfigureApp_, "ctx_")) {
                        saveAppRules();
                    }
                }
                selectionMade = true;
            }

            if (selectionMade) {
                isSelectingAppMode_ = false;
                ic->inputPanel().reset();
                ic->updateUserInterface(UserInterfaceComponent::InputPanel);
                auto* state = ic->propertyFor(&factory_);

                if (selectedMode != std::nullopt) {
                    state->commitBuffer();
                    state->reset();
                    setMode(selectedMode.value(), ic);
                    if (selectedMode == LilypadMode::Emoji) {
                        state->updateEmojiPreedit();
                    }
                }
            }
            return;
        }

        if (!keyEvent.isRelease() && !config_.cycleModeKey->empty() && keyEvent.key().checkKeyList(*config_.cycleModeKey)) {
            LILYPAD_INFO("Cycle mode key pressed");
            std::string                               appName  = getProgramName(ic);
            LilypadMode                               realMode = getAppRule(appName);

            auto                                      order      = stringutils::split(*config_.modeOrder, ",");
            std::vector<std::pair<std::string, bool>> visibility = {{"Smooth", *config_.showModeSmooth},
                                                                    {"Uinput", *config_.showModeUinput},
                                                                    {"Minecraft", *config_.showModeMinecraft},
                                                                    {"SurroundingText", *config_.showModeSurroundingText},
                                                                    {"Preedit", *config_.showModePreedit},
                                                                    {"Emoji", *config_.showModeEmoji},
                                                                    {"Off", *config_.showModeOff},
                                                                    {"SuperSmooth", *config_.showModeSuperSmooth},
                                                                    {"Default", *config_.showModeDefault}};

            std::vector<LilypadMode>                  enabledModes;
            for (const auto& name : order) {
                bool visible = false;
                for (const auto& v : visibility) {
                    if (v.first == name) {
                        visible = v.second;
                        break;
                    }
                }
                if (visible) {
                    std::optional<LilypadMode> mode = std::nullopt;
                    if (name == "Smooth")
                        mode = LilypadMode::Smooth;
                    else if (name == "Uinput")
                        mode = LilypadMode::Uinput;
                    else if (name == "Minecraft")
                        mode = LilypadMode::Minecraft;
                    else if (name == "SurroundingText")
                        mode = LilypadMode::SurroundingText;
                    else if (name == "Preedit")
                        mode = LilypadMode::Preedit;
                    else if (name == "Emoji")
                        mode = LilypadMode::Emoji;
                    else if (name == "Off")
                        mode = LilypadMode::Off;
                    else if (name == "SuperSmooth")
                        mode = LilypadMode::SuperSmooth;
                    else if (name == "Sequence")
                        mode = LilypadMode::Sequence;
                    else if (name == "Default")
                        mode = config().mode.value();
                    else
                        continue;

                    bool duplicate = false;
                    for (auto m : enabledModes) {
                        if (m == mode) {
                            duplicate = true;
                            break;
                        }
                    }
                    if (!duplicate) {
                        enabledModes.push_back(mode.value());
                    }
                }
            }

            if (!enabledModes.empty()) {
                size_t currentIdx = 0;
                bool   found      = false;
                for (size_t i = 0; i < enabledModes.size(); ++i) {
                    if (enabledModes[i] == realMode) {
                        currentIdx = i;
                        found      = true;
                        break;
                    }
                }

                LilypadMode nextMode = found ? enabledModes[(currentIdx + 1) % enabledModes.size()] : enabledModes[0];
                setMode(nextMode, ic);
                setAppRule(appName, nextMode);
                showCycleModeNotification(nextMode, ic);
            }

            keyEvent.filterAndAccept();
            return;
        }

        if (!keyEvent.isRelease() && !config_.modeMenuKey->empty() && keyEvent.key().checkKeyList(*config_.modeMenuKey)) {
            LILYPAD_INFO("Mode menu key pressed");
            currentConfigureApp_ = getProgramName(ic);
            g_mouse_clicked.store(false, std::memory_order_release);
            std::string appName = getProgramName(ic);
            setMode(getAppRule(appName), ic);
            showAppModeMenu(ic);
            keyEvent.filterAndAccept();
            return;
        }
        auto* state = keyEvent.inputContext()->propertyFor(&factory_);
        state->keyEvent(keyEvent);
        const auto&  s       = ic->surroundingText();
        const auto&  text    = s.text();
        size_t       textLen = fcitx_utf8_strlen(text.c_str());
        unsigned int cursor  = s.cursor();
        if (textLen == static_cast<size_t>(cursor))
            realtextLen.store(static_cast<unsigned int>(textLen), std::memory_order_release);
    }

    void LilypadEngine::reset(const InputMethodEntry& /*entry*/, InputContextEvent& event) {
        LILYPAD_INFO("Reset engine");
        auto* state = event.inputContext()->propertyFor(&factory_);
        if (!state->isEmptyHistory() && event.type() != EventType::InputContextFocusOut) {
            return;
        }

        if (event.type() == EventType::InputContextFocusOut || event.type() == EventType::InputContextReset) {
            state->reset(event.type() == EventType::InputContextFocusOut);
        }
    }

    void LilypadEngine::deactivate(const InputMethodEntry& /*entry*/, InputContextEvent& event) {
        auto*      ic              = event.inputContext();
        auto*      state           = ic->propertyFor(&factory_);
        const bool surrvalid       = ic->surroundingText().isValid();
        const bool is_dbus         = getFrontendName(ic) == "dbus";
        state->lastDeactivateTime_ = now_ms();
        if (realMode == LilypadMode::Preedit && event.type() != EventType::InputContextFocusOut) {
            state->commitBuffer();
        } else {
            std::string appName = getProgramName(ic);
            if ((event.type() == EventType::InputContextFocusOut && is_dbus && !surrvalid) || appName == "ONLYOFFICE") {
                state->lastDeactivateTime_ = now_ms();
                LILYPAD_INFO("Skip clearAllBuffers for " + appName);
            } else {
                if (surrvalid && state->oldPreBuffer_.empty())
                    state->clearAllBuffers();
            }
            is_deleting_.store(false);
            needEngineReset.store(false);
            ic->inputPanel().reset();
            ic->updateUserInterface(UserInterfaceComponent::InputPanel);
            if (realMode == LilypadMode::Preedit || realMode == LilypadMode::Emoji || realMode == LilypadMode::SurroundingText)
                ic->updatePreedit();
        }
    }

    void LilypadEngine::refreshEngine() {
        if (!factory_.registered())
            return;
        instance_->inputContextManager().foreach ([this](InputContext* ic) {
            auto* state = ic->propertyFor(&factory_);
            state->setEngine();
            if (ic->hasFocus())
                state->reset();
            return true;
        });
    }

    void LilypadEngine::refreshOption() {
        if (!factory_.registered())
            return;
        instance_->inputContextManager().foreach ([this](InputContext* ic) {
            auto* state = ic->propertyFor(&factory_);
            state->setOption();
            if (ic->hasFocus())
                state->reset();
            return true;
        });
    }

    void LilypadEngine::updateCharsetAction(InputContext* ic) {
        auto name = stringutils::concat(CharsetActionPrefix, *config_.outputCharset);
        for (const auto& action : charsetSubAction_) {
            action->setChecked(action->name() == name);
            if (ic != nullptr)
                action->update(ic);
        }
    }

    void LilypadEngine::loadAppRules() {
        {
            std::lock_guard<std::mutex>                  lock(appRulesMutex_);
            std::unordered_map<std::string, LilypadMode> ctxRules;
            for (const auto& [app, mode] : appRules_) {
                if (isStartsWith(app, "ctx_")) {
                    ctxRules[app] = mode;
                }
            }
            appRules_ = std::move(ctxRules);
        }
        auto loadFromFile = [this](const std::string& path) {
            if (path.empty()) {
                LILYPAD_WARN("App rules path is empty, skipping load");
                return;
            }
            std::ifstream file(path);
            if (!file.is_open())
                return;

            std::unordered_map<std::string, LilypadMode> tempRules;
            std::string                                  line;
            while (std::getline(file, line)) {
                if (line.empty() || line[0] == '#')
                    continue;
                auto delimiterPos = line.find('=');
                if (delimiterPos != std::string::npos) {
                    std::string app  = line.substr(0, delimiterPos);
                    std::string mode = line.substr(delimiterPos + 1);
                    try {
                        tempRules[app] = intToMode(std::stoi(mode));
                    } catch (const std::exception&) { LILYPAD_WARN("Invalid mode value for app: " + app); }
                }
            }
            file.close();

            std::lock_guard<std::mutex> lock(appRulesMutex_);
            for (const auto& [app, mode] : tempRules) {
                appRules_[app] = mode;
            }
        };
        loadFromFile(appRulesPath_);

        std::lock_guard<std::mutex> lock(appRulesMutex_);
        std::vector<lilypadAppRule> rules;
        for (const auto& pair : appRules_) {
            if (pair.first.find("ctx_") == 0)
                continue;
            lilypadAppRule rule;
            rule.app.setValue(pair.first);
            rule.mode.setValue(modeToInt(pair.second));
            rules.push_back(std::move(rule));
        }
        appRulesTables_.rules.setValue(std::move(rules));
    }

    void LilypadEngine::saveAppRules() const {
        // Method is const but locks mutable appRulesMutex_ to safely read appRules_ state
        std::ofstream file(appRulesPath_, std::ios::trunc);
        if (!file.is_open())
            return;

        file << "# Lilypad Per-App Configuration\n";
        file << "# 0 = Off, 1 = Uinput (Smooth), 2 = Uinput (Slow), 3 = Uinput (Hardcore), 4 = Surrounding Text, 5 = Preedit, 6 = Emoji Picker\n";
        std::lock_guard<std::mutex> lock(appRulesMutex_);
        for (const auto& pair : appRules_) {
            bool currentIsCtx = isStartsWith(pair.first, "ctx_");
            if (!currentIsCtx) {
                file << pair.first << "=" << modeToInt(pair.second) << "\n";
            }
        }
        file.close();
    }

    LilypadMode LilypadEngine::getAppRule(const std::string& appName) const {
        std::lock_guard<std::mutex> lock(appRulesMutex_);
        auto                        it = appRules_.find(appName);
        if (it != appRules_.end()) {
            return it->second;
        }
        return config_.mode.value();
    }

    void LilypadEngine::setAppRule(const std::string& appName, LilypadMode mode) {
        auto rules = *appRulesTables_.rules;

        bool found = false;
        for (auto& rule : rules) {
            if (*rule.app == appName) {
                rule.mode.setValue(modeToInt(mode));
                found = true;
                break;
            }
        }

        if (!found) {
            lilypadAppRule newRule;
            newRule.app.setValue(appName);
            newRule.mode.setValue(modeToInt(mode));
            rules.push_back(std::move(newRule));
        }

        {
            std::lock_guard<std::mutex> lock(appRulesMutex_);
            appRules_[appName] = mode;
        }
        appRulesTables_.rules.setValue(std::move(rules));
    }

    void LilypadEngine::closeAppModeMenu() {
        isSelectingAppMode_ = false;
        g_mouse_clicked.store(false, std::memory_order_release);
    }

    void LilypadEngine::showAppModeMenu(InputContext* ic) {
        isSelectingAppMode_ = true;

        auto candidateList = std::make_unique<CommonCandidateList>();

        candidateList->setLayoutHint(CandidateLayoutHint::Vertical);
        candidateList->setPageSize(10);

        auto getLabel = [&](const LilypadMode& modeName, const std::string& modeLabel) {
            if (modeName == realMode) {
                return Text(">> " + modeLabel);
            }
            return Text("   " + modeLabel);
        };

        auto cleanup = [this](InputContext* ic) {
            isSelectingAppMode_ = false;
            ic->inputPanel().reset();
            ic->updateUserInterface(UserInterfaceComponent::InputPanel);
            auto* state = ic->propertyFor(&factory_);
            state->commitBuffer();
            state->reset();
        };

        auto applyMode = [this, cleanup](LilypadMode mode) {
            return [this, mode, cleanup](InputContext* ic) {
                if (mode != LilypadMode::Emoji) {
                    setAppRule(currentConfigureApp_, mode);
                    if (!isStartsWith(currentConfigureApp_, "ctx_")) {
                        saveAppRules();
                    }
                }

                cleanup(ic);
                setMode(mode, ic);
                if (mode == LilypadMode::Emoji) {
                    auto* state = ic->propertyFor(&factory_);
                    state->updateEmojiPreedit();
                }
            };
        };

        struct ModeInfo {
            LilypadMode mode;
            std::string label;
            KeySym      key;
            bool        visible;
        };

        auto                                      getShortcut = [](const std::string& shortcut) { return Key(shortcut).sym(); };

        std::unordered_map<std::string, ModeInfo> modeMap = {
            {"Smooth", {LilypadMode::Smooth, _("Uinput (Smooth)"), getShortcut(*config_.shortcutSmooth), *config_.showModeSmooth}},
            {"Uinput", {LilypadMode::Uinput, _("Uinput (Slow)"), getShortcut(*config_.shortcutUinput), *config_.showModeUinput}},
            {"Minecraft", {LilypadMode::Minecraft, _("Minecraft"), getShortcut(*config_.shortcutMinecraft), *config_.showModeMinecraft}},
            {"SurroundingText", {LilypadMode::SurroundingText, _("Surrounding Text"), getShortcut(*config_.shortcutSurroundingText), *config_.showModeSurroundingText}},
            {"Preedit", {LilypadMode::Preedit, _("Preedit"), getShortcut(*config_.shortcutPreedit), *config_.showModePreedit}},
            {"Emoji", {LilypadMode::Emoji, _("Emoji Picker"), getShortcut(*config_.shortcutEmoji), *config_.showModeEmoji}},
            {"Off", {LilypadMode::Off, _("OFF"), getShortcut(*config_.shortcutOff), *config_.showModeOff}},
            {"SuperSmooth", {LilypadMode::SuperSmooth, _("Uinput (Super Smooth)"), getShortcut(*config_.shortcutSuperSmooth), *config_.showModeSuperSmooth}},
            {"Sequence", {LilypadMode::Sequence, _("Sequence"), getShortcut(*config_.shortcutSequence), *config_.showModeSequence}},
            {"Default", {config_.mode.value(), _("Default Typing"), getShortcut(*config_.shortcutDefault), *config_.showModeDefault}}};

        std::vector<ModeInfo> allModes;
        auto                  order = stringutils::split(*config_.modeOrder, ",");
        for (const auto& name : order) {
            auto it = modeMap.find(name);
            if (it != modeMap.end()) {
                allModes.push_back(it->second);
            }
        }

        // Fallback for missing modes
        for (const auto& [name, info] : modeMap) {
            bool found = false;
            for (const auto& orderedName : order) {
                if (orderedName == name) {
                    found = true;
                    break;
                }
            }
            if (!found) {
                allModes.push_back(info);
            }
        }

        int                        activeSelectionIdx  = -1;
        int                        currentCandidateIdx = 0;
        std::unordered_set<KeySym> usedModeKeys;

        modeMenuMapping_.clear();
        const LilypadMode defaultMode = config_.mode.value();

        for (const auto& info : allModes) {
            if (info.visible) {
                const bool hasShortcut = info.key != FcitxKey_None && info.key != FcitxKey_VoidSymbol;
                if (hasShortcut && usedModeKeys.insert(info.key).second) {
                    modeMenuMapping_[info.key] = info.mode;
                }

                const std::string keyUtf8  = Key::keySymToUTF8(info.key);
                std::string       keyLabel = keyUtf8.empty() ? "" : "[" + keyUtf8 + "] ";
                candidateList->append(std::make_unique<AppModeCandidateWord>(getLabel(info.mode, keyLabel + info.label), applyMode(info.mode)));

                if (info.mode == realMode) {
                    activeSelectionIdx = currentCandidateIdx;
                } else if (info.mode == defaultMode && info.label == _("Default Typing") && getAppRule(currentConfigureApp_) == defaultMode) {
                    // This is technically tricky because getAppRule returns the global default if no rule exists.
                    // If we are at global default, highlight "Default Typing".
#if __cplusplus >= 202002L
                    if (!appRules_.contains(currentConfigureApp_)) {
#else
                    if (appRules_.find(currentConfigureApp_) == appRules_.end()) {
#endif
                        activeSelectionIdx = currentCandidateIdx;
                    }
                }
                currentCandidateIdx++;
            }
        }

        {
            const auto& kl = *config_.modeMenuKey;
            if (kl.size() == 1 && !kl[0].hasModifier()) {
                std::string charStr = Key::keySymToUTF8(kl[0].sym());
                if (!charStr.empty()) {
                    KeySym      typeKeySym   = typeKeyForModeMenuHotkey(kl[0].sym(), config_);
                    std::string typeKeyLabel = Key::keySymToUTF8(typeKeySym);
                    std::string label        = "[" + typeKeyLabel + "] " + _("Type") + " " + charStr;
                    candidateList->append(std::make_unique<AppModeCandidateWord>(Text(label), [cleanup, charStr](InputContext* ic) {
                        cleanup(ic);
                        ic->commitString(charStr);
                    }));
                }
            }
        }

        if (activeSelectionIdx != -1) {
            candidateList->setGlobalCursorIndex(activeSelectionIdx);
        } else if (candidateList->totalSize() > 0) {
            candidateList->setGlobalCursorIndex(0);
        }

        ic->inputPanel().reset();
        ic->inputPanel().setCandidateList(std::move(candidateList));
        ic->inputPanel().setAuxDown(Text(_("App: ") + currentConfigureApp_));
        ic->updateUserInterface(UserInterfaceComponent::InputPanel);
    }

    void LilypadEngine::showCycleModeNotification(LilypadMode mode, InputContext* ic) {
        auto candidateList = std::make_unique<CommonCandidateList>();
        candidateList->setLayoutHint(CandidateLayoutHint::Vertical);
        candidateList->setPageSize(1);

        auto cleanup = [](InputContext* ic) {
            ic->inputPanel().reset();
            ic->updateUserInterface(UserInterfaceComponent::InputPanel);
        };

        // Map mode to label
        std::string modeLabel;
        switch (mode) {
            case LilypadMode::Smooth: modeLabel = _("Uinput (Smooth)"); break;
            case LilypadMode::Uinput: modeLabel = _("Uinput (Slow)"); break;
            case LilypadMode::Minecraft: modeLabel = _("Minecraft"); break;
            case LilypadMode::SurroundingText: modeLabel = _("Surrounding Text"); break;
            case LilypadMode::Preedit: modeLabel = _("Preedit"); break;
            case LilypadMode::Emoji: modeLabel = _("Emoji Picker"); break;
            case LilypadMode::Off: modeLabel = _("OFF"); break;
            case LilypadMode::SuperSmooth: modeLabel = _("Uinput (Super Smooth)"); break;
            case LilypadMode::Sequence: modeLabel = _("Sequence"); break;
            default: modeLabel = _("Unknown Mode"); break;
        }

        auto setCurrentMode = [cleanup](LilypadMode) { return [cleanup](InputContext* ic) { cleanup(ic); }; };

        candidateList->append(std::make_unique<AppModeCandidateWord>(Text("✓ " + modeLabel), setCurrentMode(mode)));

        candidateList->setGlobalCursorIndex(0);

        ic->inputPanel().reset();
        ic->inputPanel().setCandidateList(std::move(candidateList));
        ic->updateUserInterface(UserInterfaceComponent::InputPanel);

        // Cancel previous timer if any
        cycleModeNotificationTimer_.reset();

        // Schedule auto-close using EventLoop::addTimeEvent
        auto& eventLoop    = instance_->eventLoop();
        auto  now_time     = ::fcitx::now(CLOCK_MONOTONIC);
        auto  timeout_time = now_time + CYCLE_MODE_NOTIFICATION_TIMEOUT_USEC;

        cycleModeNotificationTimer_ = eventLoop.addTimeEvent(CLOCK_MONOTONIC, timeout_time, 0, [icRef = ic->watch()](EventSourceTime*, uint64_t) {
            if (auto* ic = icRef.get(); ic && ic->hasFocus()) {
                ic->inputPanel().reset();
                ic->updateUserInterface(UserInterfaceComponent::InputPanel);
            }
            return false;
        });
    }

    void LilypadEngine::setMode(LilypadMode mode, InputContext* ic) {
        realMode = mode;
        if (ic != nullptr) {
            if (auto* state = ic->propertyFor(&factory_)) {
                state->clearAllBuffers();
            }
            ic->updateUserInterface(UserInterfaceComponent::StatusArea);
        }
    }

    std::string LilypadEngine::subModeIconImpl(const InputMethodEntry& /*entry*/, InputContext& /*inputContext*/) {
        std::string baseIconName;
        switch (realMode) {
            case LilypadMode::Off: baseIconName = "fcitx-lilypad-off"; break;
            case LilypadMode::Emoji: baseIconName = "fcitx-lilypad-emoji"; break;
            default: baseIconName = "fcitx-lilypad"; break;
        }

        if (*config_.useLilypadIcons) {
            return baseIconName;
        }

        const auto& iconTheme = config_.iconTheme.value();
        if (iconTheme == IconTheme::Light) {
            return baseIconName + "-default-black";
        }
        if (iconTheme == IconTheme::Dark) {
            return baseIconName + "-default";
        }

        return baseIconName + (isDarkMode() ? "-default" : "-default-black");
    }

    std::string LilypadEngine::subModeLabelImpl(const InputMethodEntry& /*entry*/, InputContext& /*inputContext*/) {
        switch (realMode) {
            case LilypadMode::Off: return _("Lilypad - Off");
            case LilypadMode::Emoji: return "😄";
            default: return isGnome_ ? "vi" : "🍃";
        }
    }

    std::string LilypadEngine::getProgramName(InputContext* ic) {
        if (ic == nullptr) {
            return "unknown-app";
        }
        std::string programName = ic->program();
        std::string lower = programName;
#if __cplusplus >= 202002L
        std::ranges::transform(lower, lower.begin(), ::tolower);
#else
        std::transform(lower.begin(), lower.end(), lower.begin(), ::tolower);
#endif
        if (lower.find("onlyoffice") != std::string::npos ||
            lower.find("desktopeditors") != std::string::npos ||
            lower.find("editors_helper") != std::string::npos) {
            return "ONLYOFFICE";
        }

        if (programName.empty() || programName == "wayland" || programName == "x11") {
            // Fallback: InputContext address-based resolution
            // This ensures at least per-window separation.
            std::ostringstream oss;
            oss << "ctx_" << static_cast<const void*>(ic);
            programName = oss.str();
        }
        return programName;
    }
} // namespace fcitx
