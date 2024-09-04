#include <logger/file_logger.h>

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <stdarg.h>
#include <time.h>

#define MAX_LOG_FILES 5
#define MAX_FILE_SIZE 1024 * 1024

#define LOG_FILENAME "/var/log/wallmon.log"

static void rotate_logs(const char *base_filename) {
    char old_name[256], new_name[256];

    snprintf(old_name, sizeof(old_name), "%s.%d", base_filename, MAX_LOG_FILES - 1);
    remove(old_name);

    for (int i = MAX_LOG_FILES - 2; i >= 0; i--) {
        snprintf(old_name, sizeof(old_name), "%s.%d", base_filename, i);
        snprintf(new_name, sizeof(new_name), "%s.%d", base_filename, i + 1);
        rename(old_name, new_name);
    }

    snprintf(new_name, sizeof(new_name), "%s.%d", base_filename, 0);
    rename(base_filename, new_name);

    FILE *log_file = fopen(base_filename, "w");
    if (log_file) {
        fclose(log_file);
    } else {
        perror("Failed to truncate log file");
    }
}

static void write_log_label(FILE *stream, const char *severity) {
    time_t     timestamp = time(NULL);
    struct tm *time      = gmtime(&timestamp);

    char   buffer[32];
    size_t len = strftime(buffer, sizeof(buffer), "%Y-%m-%dT%H:%M:%SZ", time);

    if (len > 0 && len < sizeof(buffer)) {
        fprintf(stream, "[%s : %s]\n", buffer, severity);
    } else {
        fprintf(stream, "[%ld : %s]\n", timestamp, severity);
    }
}

static void log_message_common(const char *base_filename, const char *sev, const char *format, va_list args) {
    FILE *log_file = fopen(base_filename, "a+");
    if (!log_file) {
        perror("Failed to open log file");
        return;
    }

    fseek(log_file, 0, SEEK_END);
    long file_size = ftell(log_file);
    fclose(log_file);

    if (file_size >= MAX_FILE_SIZE) {
        rotate_logs(base_filename);
    }

    log_file = fopen(base_filename, "a+");
    if (!log_file) {
        perror("Failed to reopen log file");
        return;
    }

    write_log_label(log_file, sev);
    vfprintf(log_file, format, args);
    fprintf(log_file, "\n");

    fflush(log_file);
    fclose(log_file);
}

void vfl_log_message(log_severity severity, const char *format, va_list args) {
    switch (severity) {
        case LOG_SEVERITY_INFO:
            log_message_common(LOG_FILENAME, "INFO", format, args);
            break;

        case LOG_SEVERITY_WARN:
            log_message_common(LOG_FILENAME, "WARN", format, args);
            break;

        case LOG_SEVERITY_ERROR:
            log_message_common(LOG_FILENAME, "ERROR", format, args);
            break;

        default:
            break;
    }
}
