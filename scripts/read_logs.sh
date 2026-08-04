#!/usr/bin/env bash
# Trình đọc log thời gian thực cho vnlilypad-lotus / fcitx5
echo "🚀 Đang khởi chạy Trình đọc log thời gian thực (vnlilypad-lotus)..."
FCITX_LOG_LEVEL=debug fcitx5 -r 2>&1 | grep --line-buffered -E "LILYPAD|LOTUS|AT-SPI|SEQUENCER|WAYLAND"
