#ifndef _LOGGER_LOGGER_H_
#define _LOGGER_LOGGER_H_

#include <logger/common.h>

/**
 * @brief Initializes the logger system with a specified type(s) and severity level.
 *
 * @param ident    Logger identification.
 * @param type     The output destination(s) for the logs, specified using the logger type flag.
 * @param severity The minimum severity level of messages that will be logged.
 *
 */
void logger_init(const char* ident, logger_type_flag type, log_severity severity);

/**
 * @brief Cleans up the logger system.
 */
void logger_cleanup();

/**
 * @brief Logs a message with a specified severity level using variadic arguments.
 *
 * @param severity The severity level of the log message.
 * @param format   The format string for the log message, following `printf`-style syntax.
 * @param ...      Variadic arguments that are used in the format string.
 */
void _log_message(log_severity severity, const char* format, ...);

/**
 * @brief Macro to log a message with a specified severity level and variadic arguments.
 *
 * @param severity The severity level of the log message.
 * @param message  The format string for the log message.
 * @param ...      Additional variadic arguments for the format string.
 */
#define WALLMON_LOG_EX(severity, message, ...) _log_message(severity, message, ##__VA_ARGS__)

/**
 * @brief Macro to log an informational message with variadic arguments.
 *
 * @param message The format string for the log message.
 * @param ...     Additional variadic arguments for the format string.
 */
#define WALLMON_LOG_INFO(message, ...) WALLMON_LOG_EX(LOG_SEVERITY_INFO, message, ##__VA_ARGS__)

/**
 * @brief Macro to log a warning message with variadic arguments.
 *
 * @param message The format string for the log message.
 * @param ...     Additional variadic arguments for the format string.
 */
#define WALLMON_LOG_WARN(message, ...) WALLMON_LOG_EX(LOG_SEVERITY_WARN, message, ##__VA_ARGS__)

/**
 * @brief Macro to log an error message with variadic arguments.
 *
 * @param message The format string for the log message.
 * @param ...     Additional variadic arguments for the format string.
 */
#define WALLMON_LOG_ERROR(message, ...) WALLMON_LOG_EX(LOG_SEVERITY_ERROR, message, ##__VA_ARGS__)

#endif
