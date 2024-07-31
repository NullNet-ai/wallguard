#include <utils/file_utils.h>

#include <unistd.h>

int file_exists(const char *path) { return access(path, F_OK) == 0; }

int files_exist(const char *files[], int count) {
    for (int i = 0; i < count; ++i) {
        if (!file_exists(files[i])) {
            return 0;
        }
    }

    return 1;
}