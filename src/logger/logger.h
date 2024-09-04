#ifndef _LOGGER_LOGGER_H_
#define _LOGGER_LOGGER_H_

#include <logger/common.h>

/**
 * @brief Logs a message with a specified severity level using variadic arguments.
 *
 * @param severity The severity level of the log message.
 * @param format   The format string for the log message, following `printf`-style syntax.
 * @param ...      Variadic arguments that are used in the format string.
 */
void _log_message(log_severity severity, const char *format, ...);

/**
 * @def WLOG_SET_LOG_LEVEL(severity)
 * @brief Macro to set the global log severity level.
 *
 * @param severity The severity level to set.
 */
#define WLOG_SET_LOG_LEVEL(severity) set_log_level(severity)

/**
 * @def WLOG_SET_TYPE_FLAG(flag)
 * @brief Macro to set the logger type flag(s) for output destinations.
 *
 * @param flag The logger type flag(s) to set.
 */
#define WLOG_SET_TYPE_FLAG(flag) set_logger_type_flag(flag)

/**
 * @def WLOG_EX(severity, message, ...)
 * @brief Macro to log a message with a specified severity level and variadic arguments.
 *
 * @param severity The severity level of the log message.
 * @param message  The format string for the log message.
 * @param ...      Additional variadic arguments for the format string.
 */
#define WLOG_EX(severity, message, ...) _log_message(severity, message, ##__VA_ARGS__)

/**
 * @def WLOG_INFO(message, ...)
 * @brief Macro to log an informational message with variadic arguments.
 *
 * @param message The format string for the log message.
 * @param ...     Additional variadic arguments for the format string.
 */
#define WLOG_INFO(message, ...) WLOG_EX(LOG_SEVERITY_INFO, message, ##__VA_ARGS__)

/**
 * @def WLOG_WARN(message, ...)
 * @brief Macro to log a warning message with variadic arguments.
 *
 * @param message The format string for the log message.
 * @param ...     Additional variadic arguments for the format string.
 */
#define WLOG_WARN(message, ...) WLOG_EX(LOG_SEVERITY_WARN, message, ##__VA_ARGS__)

/**
 * @def WLOG_ERROR(message, ...)
 * @brief Macro to log an error message with variadic arguments.
 *
 * @param message The format string for the log message.
 * @param ...     Additional variadic arguments for the format string.
 */
#define WLOG_ERROR(message, ...) WLOG_EX(LOG_SEVERITY_ERROR, message, ##__VA_ARGS__)

#endif
