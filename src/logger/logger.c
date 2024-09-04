#include <logger/logger.h>

#include <logger/console_logger.h>
#include <logger/file_logger.h>

#include <stdarg.h>

void _log_message(log_severity severity, const char *format, ...) {
    if (severity < log_level) {
        return;
    }

    va_list args;
    va_start(args, format);

    if (logger_type_flags & LOGGER_TYPE_FILE) {
        va_list args_copy;
        va_copy(args_copy, args);
        vfl_log_message(severity, format, args_copy);
    }

    if (logger_type_flags & LOGGER_TYPE_CONSOLE) {
        va_list args_copy;
        va_copy(args_copy, args);
        vcl_log_message(severity, format, args_copy);
    }

    va_end(args);
}