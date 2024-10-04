#include "syslog_logger.h"

#define __USE_MISC
#include <syslog.h>

void vsl_log_message(log_severity severity, const char* format, va_list args) {
    switch (severity) {
        case LOG_SEVERITY_INFO:
            vsyslog(LOG_NOTICE, format, args);
            break;

        case LOG_SEVERITY_WARN:
            vsyslog(LOG_WARNING, format, args);
            break;

        case LOG_SEVERITY_ERROR:
            vsyslog(LOG_ERR, format, args);
            break;
        default:
            break;
    }
}

void vsl_init(const char* ident) { openlog(ident, LOG_PID, LOG_USER); }

void vsl_cleanup() { closelog(); }
