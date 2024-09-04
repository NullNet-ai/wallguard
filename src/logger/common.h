#ifndef _LOGGER_COMMON_H_
#define _LOGGER_COMMON_H_

/**
 * @enum log_severity
 * @brief Represents the severity level of log messages.
 */
typedef enum {
    LOG_SEVERITY_INFO,
    LOG_SEVERITY_WARN,
    LOG_SEVERITY_ERROR,
} log_severity;

/**
 * @enum logger_type
 * @brief Represents the type of logger output.
 */
typedef enum {
    LOGGER_TYPE_CONSOLE = 1 << 0,
    LOGGER_TYPE_FILE    = 1 << 1,
} logger_type_flag;

/**
 * @var log_level
 * @brief Global log severity level.
 */
extern log_severity log_level;

/**
 * @brief Sets the global log severity level.
 *
 * @param level The severity level to set.
 */
void set_log_level(log_severity level);

/**
 * @var logger_type_flags
 * @brief Global logger output flags.
 */
extern logger_type_flag logger_type_flags;

/**
 * @brief Sets the logger type flag(s) for log output.
 *
 * @param flag The logger type flag(s) to set (e.g., LOGGER_TYPE_CONSOLE or LOGGER_TYPE_FILE).
 */
void set_logger_type_flag(logger_type_flag flag);

#endif
