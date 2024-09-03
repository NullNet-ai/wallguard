#include <platform/bootstrap.h>
#include <utils/file_utils.h>

#include <sys/stat.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>

#define LOCK_FOLDER "/var/lock/"
#define LOCK_FILENAME "wallmon.lock"

boolean_t system_locked(void) { return file_exists(LOCK_FOLDER LOCK_FILENAME); }

boolean_t lock_system(platform_info* info) {
    if (!directory_exists(LOCK_FOLDER) && (mkdir(LOCK_FOLDER, 0777) != 0)) {
        return WM_FALSE;
    }

    FILE* lockfile = fopen(LOCK_FOLDER LOCK_FILENAME, "w");

    if (!lockfile) {
        return WM_FALSE;
    }

    size_t len    = ARRAY_SIZE(info->uuid);
    size_t result = fwrite(info->uuid, 1, len, lockfile);

    fclose(lockfile);
    return len == result;
}

boolean_t validate_lock(platform_info* info) {
    FILE* lockfile = fopen(LOCK_FOLDER LOCK_FILENAME, "r");

    if (!lockfile) {
        return WM_FALSE;
    }

    char buffer[sizeof(info->uuid)] = {0};

    size_t read_len = fread(buffer, 1, sizeof(info->uuid), lockfile);
    fclose(lockfile);

    if (read_len != sizeof(info->uuid)) {
        return WM_FALSE;
    }

    return strncmp(info->uuid, buffer, sizeof(info->uuid)) == 0;
}
