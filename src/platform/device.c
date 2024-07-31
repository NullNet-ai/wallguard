#include <platform/device.h>
#include <utils/file_utils.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(__FreeBSD__)
#define DMIDECODE_PATH "/usr/local/sbin/dmidecode"
#else
#define DMIDECODE_PATH "/usr/sbin/dmidecode"
#endif

static int dmidecode_available() { return file_exists(DMIDECODE_PATH); }

static int dmidecode_uuid(char* uuid, size_t size) {
    int   retval = 1;
    FILE* pipe   = NULL;

    pipe = popen("dmidecode -s system-uuid", "r");
    if (!pipe) {
        retval = 0;
        goto __exit;
    }

    if (fgets(uuid, size, pipe) == NULL) {
        retval = 0;
        goto __exit;
    }

    size_t len = strcspn(uuid, "\n");
    if (len < size) {
        uuid[len] = '\0';
    }

__exit:
    if (pipe) {
        pclose(pipe);
    }
    return retval;
}

int device_uuid(char* uuid, size_t size) {
    // @TODO: check buffer and size and ensure null-ternmiantion
    if (dmidecode_available()) {
        return dmidecode_uuid(uuid, size);
    }

    return 0;
}