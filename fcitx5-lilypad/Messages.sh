#!/bin/bash

xgettext \
--language=C++ \
--from-code=UTF-8 \
--keyword=_ \
--keyword=N_ \
-o /tmp/lilypad-cpp.pot \
$(find . \( -name "*.cpp" -o -name "*.h" \))

xgettext \
--language=appdata \
--from-code=UTF-8 \
-o /tmp/lilypad-xml.pot \
org.fcitx.Fcitx5.Addon.Lilypad.metainfo.xml.in.in

xgettext \
--language=Python \
--from-code=UTF-8 \
--keyword=_ \
--keyword=N_ \
-o /tmp/lilypad-python.pot \
$(find . -name "*.py")

xgettext \
--language=Desktop \
--from-code=UTF-8 \
--keyword=Name \
--keyword=Comment \
-o /tmp/lilypad-desktop.pot \
settings-gui/org.fcitx.Fcitx5.Addon.Lilypad.Settings.desktop.in

{
    echo 'msgid ""'
    echo 'msgstr ""'
    echo '"Content-Type: text/plain; charset=UTF-8\n"'
    echo ""
    grep -hE "^Name=" \
    src/lilypad.conf.in \
    src/lilypad-addon.conf.in.in \
    | sed 's/^Name=\(.*\)/msgid "\1"\nmsgstr ""\n/'
} > /tmp/lilypad-conf.pot

msgcat \
--use-first \
/tmp/lilypad-cpp.pot \
/tmp/lilypad-xml.pot \
/tmp/lilypad-conf.pot \
/tmp/lilypad-python.pot \
/tmp/lilypad-desktop.pot \
-o po/fcitx5-lilypad.pot
