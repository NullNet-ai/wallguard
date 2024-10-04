#ifndef _SYSLOG_LOGGER_H_
#define _SYSLOG_LOGGER_H_

#include <logger/common.h>
#include <stdarg.h>

/**
 * @brief Logs a message to syslog.
 *
 * @param severity The severity level of the log message.
 * @param format   The format string for the log message, following `printf`-style syntax.
 * @param args     A `va_list` representing the additional arguments for the format string.
 */
void vsl_log_message(log_severity severity, const char* format, va_list args);

/**
 * @brief Initializes the syslog-based logging system.
 *
 * @param ident Logger identification
 */
void vsl_init(const char* ident);

/**
 * @brief Cleans up the syslog-based logging system.
 */
void vsl_cleanup();

#endif
