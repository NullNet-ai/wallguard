#include <logger/common.h>

log_severity log_level = LOG_SEVERITY_INFO;

void set_log_level(log_severity level) { log_level = level; }

logger_type_flag logger_type_flags = 0;

void set_logger_type_flag(logger_type_flag flag) { logger_type_flags |= flag; }
