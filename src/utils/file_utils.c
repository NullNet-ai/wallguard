#include <utils/file_utils.h>

#include <unistd.h>
#include <sys/stat.h>

boolean_t file_exists(const char *path) { return access(path, F_OK) == 0; }

boolean_t files_exist(const char *files[], size_t count) {
    for (size_t i = 0; i < count; ++i) {
        if (!file_exists(files[i])) {
            return WM_FALSE;
        }
    }

    return WM_TRUE;
}

boolean_t file_monitor_init(file_monitor *monitor, const char *filepath) {
    if (!monitor || !filepath) {
        return WM_FALSE;
    }

    monitor->filepath = filepath;

    struct stat file_stat;
    if (stat(filepath, &file_stat) == -1) {
        return WM_FALSE;
    }

    monitor->last_update = file_stat.st_mtime;
    return WM_TRUE;
}

int file_monitor_check(file_monitor *monitor) {
    if (!monitor) {
        return -1;
    }

    struct stat file_stat;
    if (stat(monitor->filepath, &file_stat) == -1) {
        return -1;
    }

    if (file_stat.st_mtime != monitor->last_update) {
        monitor->last_update = file_stat.st_mtime;
        return 1;
    }

    return 0;
}
