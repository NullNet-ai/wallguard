#include <logger/console_logger.h>

#include <time.h>
#include <stdio.h>

#define CLR_RED "\x1b[31m"
#define CLR_YELLOW "\x1b[33m"
#define CLR_MAGENTA "\x1b[35m"
#define CLR_CYAN "\x1b[36m"
#define CLR_RESET "\x1b[0m"

static void write_log_label(FILE* stream, const char* severity, const char* color) {
    time_t     tsmp = time(NULL);
    struct tm* time = gmtime(&tsmp);

    char   buffer[32];
    size_t len = strftime(buffer, sizeof(buffer), "%Y-%m-%dT%H:%M:%SZ", time);

    if (len > 0 && len < sizeof(buffer)) {
        fprintf(stream, "[" CLR_MAGENTA "%s" CLR_RESET " : %s%s%s]\n", buffer, color, severity, CLR_RESET);
    } else {
        fprintf(stream, "[" CLR_MAGENTA "%ld" CLR_RESET " : %s%s%s]\n", tsmp, color, severity, CLR_RESET);
    }
}

static void log_message_common(FILE* stream, const char* sev, const char* clr, const char* format, va_list args) {
    write_log_label(stream, sev, clr);
    vfprintf(stream, format, args);
    fprintf(stream, "\n");
}

void vcl_log_message(log_severity severity, const char* format, va_list args) {
    switch (severity) {
        case LOG_SEVERITY_INFO:
            log_message_common(stdout, "INFO", CLR_CYAN, format, args);
            break;
        case LOG_SEVERITY_WARN:
            log_message_common(stdout, "WARN", CLR_YELLOW, format, args);
            break;
        case LOG_SEVERITY_ERROR:
            log_message_common(stdout, "ERROR", CLR_RED, format, args);
            break;
        default:
            break;
    }
}
