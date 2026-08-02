/*
 * SPDX-FileCopyrightText: 2026 Nguyễn Hoàng Kỳ  <nhktmdzhg@gmail.com>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 */

/**
 * @file lilypad-logger.h
 * @brief Simple file logger with rotation for fcitx5-lilypad-server
 *
 * Features:
 * - Thread-safe logging
 * - Automatic file rotation (max 10MB, keep 5 files)
 * - Timestamp and level formatting
 * - Configurable log levels
 */

#ifndef _LILYPAD_LOGGER_H_
#define _LILYPAD_LOGGER_H_

#include <atomic>
#include <cstdint>
#include <fstream>
#include <string>
#include <mutex>

enum class LogLevel : std::uint8_t {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    NONE
};

class LilypadLogger {
  public:
    /**
     * @brief Instance constructor
     */
    static LilypadLogger& instance() {
        static LilypadLogger instance_;
        return instance_;
    }

    /**
     * @brief Destructor
     */
    ~LilypadLogger();

    // Rule of five
    LilypadLogger(const LilypadLogger&)            = delete;
    LilypadLogger& operator=(const LilypadLogger&) = delete;
    LilypadLogger(LilypadLogger&&)                 = delete;
    LilypadLogger& operator=(LilypadLogger&&)      = delete;

    /**
     * @brief Set minimum log level
     */
    void setLevel(LogLevel level);

    /**
     * @brief Check if logging is enabled for given level
     */
    bool isEnabled(LogLevel level) const;

    /**
     * @brief Log a message
     */
    void log(LogLevel level, const std::string& message);

    // Convenience methods
    void debug(const std::string& msg) {
        if (isEnabled(LogLevel::DEBUG))
            log(LogLevel::DEBUG, msg);
    }
    void info(const std::string& msg) {
        if (isEnabled(LogLevel::INFO))
            log(LogLevel::INFO, msg);
    }
    void warn(const std::string& msg) {
        if (isEnabled(LogLevel::WARN))
            log(LogLevel::WARN, msg);
    }
    void error(const std::string& msg) {
        if (isEnabled(LogLevel::ERROR))
            log(LogLevel::ERROR, msg);
    }

  private:
    /**
     * @brief Constructor
     * @param log_file Path to log file
     * @param max_size Maximum file size before rotation (bytes)
     * @param max_files Maximum number of backup files to keep
     * @param level Minimum log level to output
     */
    LilypadLogger(std::string log_file = "/tmp/fcitx5-lilypad-server.log", LogLevel level = LogLevel::INFO);

    /**
     * @brief Get current timestamp string
     */
    static std::string getTimestamp();

    /**
     * @brief Get log level string
     */
    static std::string    levelToString(LogLevel level);

    std::string           log_file_;
    std::atomic<LogLevel> level_;
    std::ofstream         file_;
    std::mutex            mutex_;
};
#endif // _LILYPAD_LOGGER_H_