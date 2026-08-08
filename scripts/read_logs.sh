#!/usr/bin/env bash
# Trình đọc log thời gian thực cho fcitx5-lilypad
echo "🚀 Đang khởi chạy Trình đọc log thời gian thực (fcitx5-lilypad)..."
FCITX_LOG_LEVEL=debug fcitx5 -r 2>&1 | grep --line-buffered -E "LILYPAD|LOTUS|AT-SPI|SEQUENCER|WAYLAND"
