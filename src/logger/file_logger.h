#ifndef _LOGGER_FILE_LOGGER_H_
#define _LOGGER_FILE_LOGGER_H_

#include <logger/common.h>
#include <stdarg.h>

/**
 * @brief Logs a message to the file.
 *
 * @param severity The severity level of the log message.
 * @param format   The format string for the log message, following `printf`-style syntax.
 * @param args     A `va_list` representing the additional arguments for the format string.
 */
void vfl_log_message(log_severity severity, const char* format, va_list args);

#endif
