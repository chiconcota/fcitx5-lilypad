/*
 * SPDX-FileCopyrightText: 2026 Nguyễn Hoàng Kỳ  <nhktmdzhg@gmail.com>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

#include "lotus-logger.h"

#include <iomanip>
#include <sstream>
#include <chrono>
#include <iostream>
#include <filesystem>
#include <utility>

#include <sys/stat.h>

LotusLogger::LotusLogger(std::string log_file, LogLevel level) : log_file_(std::move(log_file)) {
    level_.store(level);

    std::filesystem::path path(log_file_);
    if (path.has_parent_path()) {
        std::filesystem::create_directories(path.parent_path());
    }

    file_.open(log_file_, std::ios_base::app | std::ios_base::out);

    if (!file_.is_open()) {
        std::cerr << "[ERROR] Failed to open log file: " << log_file_ << '\n';
    }
}

LotusLogger::~LotusLogger() {
    if (file_.is_open()) {
        file_.close();
    }
}

void LotusLogger::setLevel(LogLevel level) {
    level_.store(level);
}

bool LotusLogger::isEnabled(LogLevel level) const {
    return level >= level_.load();
}

void LotusLogger::log(LogLevel level, const std::string& message) {
    if (!isEnabled(level) || !file_.is_open()) {
        return;
    }

    std::lock_guard<std::mutex> lock(mutex_);

    std::string                 entry = getTimestamp() + " [" + levelToString(level) + "] " + message + "\n";

    file_ << entry;
    if (level >= LogLevel::WARN) {
        file_.flush();
    }
}

std::string LotusLogger::getTimestamp() {
    auto now    = std::chrono::system_clock::now();
    auto time_t = std::chrono::system_clock::to_time_t(now);
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers)
    auto      ms = std::chrono::duration_cast<std::chrono::milliseconds>(now.time_since_epoch()) % 1000;

    struct tm buf{};
    localtime_r(&time_t, &buf);

    std::stringstream ss;
    ss << std::put_time(&buf, "%Y-%m-%d %H:%M:%S");
    ss << "." << std::setfill('0') << std::setw(3) << ms.count();
    return ss.str();
}

std::string LotusLogger::levelToString(LogLevel level) {
    switch (level) {
        case LogLevel::DEBUG: return "DEBUG";
        case LogLevel::INFO: return "INFO";
        case LogLevel::WARN: return "WARN";
        case LogLevel::ERROR: return "ERROR";
        default: return "UNKNOWN";
    }
}