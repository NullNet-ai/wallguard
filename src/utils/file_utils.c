#include <utils/file_utils.h>

#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

boolean_t file_exists(const char *path) { return access(path, F_OK) == 0; }

boolean_t directory_exists(const char *path) {
    struct stat statbuf;

    if (stat(path, &statbuf) != 0) {
        return WM_FALSE;
    }

    return S_ISDIR(statbuf.st_mode);
}

boolean_t make_directory(const char *path) {
    char temp[256] = {0};
    snprintf(temp, sizeof(temp), "%s", path);

    char *p = NULL;
    for (p = temp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            if (mkdir(temp, S_IRWXU | S_IRWXG | S_IRWXO) != 0 && errno != EEXIST) {
                return WM_FALSE;
            }
            *p = '/';
        }
    }

    if (mkdir(temp, S_IRWXU | S_IRWXG | S_IRWXO) != 0 && errno != EEXIST) {
        return WM_FALSE;
    }

    return WM_TRUE;
}

ssize_t file_size(const char *path) {
    struct stat st;
    if (stat(path, &st) == 0) {
        return st.st_size;
    } else {
        return -1;
    }
}

const char *filename(const char *path) {
    const char *slash = strrchr(path, '/');
    if (slash) {
        return slash + 1;
    } else {
        return path;
    }
}

const char *extension(const char *path) {
    const char *dot = strrchr(path, '.');

    if (dot && *(dot + 1) != '\0') {
        return dot + 1;
    } else {
        return NULL;
    }
}

boolean_t copy_file(const char *source, const char *destination) {
    if (!file_exists(source)) {
        return WM_FALSE;
    }

    FILE *source_file = fopen(source, "rb");
    if (!source_file) {
        return WM_FALSE;
    }

    FILE *destination_file = fopen(destination, "wb");
    if (!destination_file) {
        fclose(source_file);
        return WM_FALSE;
    }

    size_t  bytes;
    uint8_t buffer[1024];

    while ((bytes = fread(buffer, sizeof(buffer[0]), sizeof(buffer), source_file)) > 0) {
        fwrite(buffer, sizeof(buffer[0]), bytes, destination_file);
    }

    fclose(source_file);
    fclose(destination_file);

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

// #TODO: Do i need this function ?
uint8_t *read_file_content(const char *path) {
    FILE *file = fopen(path, "rb");
    if (!file) {
        return NULL;
    }

    struct stat buffer;
    memset(&buffer, 0, sizeof(struct stat));

    if (stat(path, &buffer) != 0) {
        fclose(file);
        return NULL;
    }

    uint8_t *content = W_MALLOC(buffer.st_size + 1);

    size_t rb = fread(content, sizeof(content[0]), buffer.st_size, file);
    (void)rb;
    fclose(file);

    content[buffer.st_size] = '\0';
    return content;
}
