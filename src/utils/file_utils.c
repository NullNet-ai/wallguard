#include <utils/file_utils.h>

#include <unistd.h>
#include <sys/stat.h>

int file_exists(const char *path) { return access(path, F_OK) == 0; }

int files_exist(const char *files[], size_t count) {
    for (size_t i = 0; i < count; ++i) {
        if (!file_exists(files[i])) {
            return 0;
        }
    }

    return 1;
}

int file_monitor_init(file_monitor *monitor, const char *filepath) {
    if (!monitor || !filepath) {
        return -1;
    }

    monitor->filepath = filepath;

    struct stat file_stat;
    if (stat(filepath, &file_stat) == -1) {
        return -1;
    }

    monitor->last_update = file_stat.st_mtime;
    return 0;
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
