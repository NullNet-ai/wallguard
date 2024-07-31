#include <platform/device.h>
#include <utils/file_utils.h>
#include <utils/common.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int dmidecode_available() {
#if defined(__FreeBSD__)
    const char* dmidecode_path = "/usr/local/sbin/dmidecode";
#else
    const char* dmidecode_path = "/usr/sbin/dmidecode";
#endif
    return file_exists(dmidecode_path);
}

static int dmidecode_uuid(char* uuid, size_t size) {
    int   retval = 1;
    FILE* pipe   = popen("dmidecode -s system-uuid", "r");

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

#if defined(__FreeBSD__)
#include <sys/types.h>
#include <sys/sysctl.h>

static int systctl_uuid(char* uuid, size_t size) {
    int mib[2];

    mib[0] = CTL_KERN;
    mib[1] = KERN_HOSTUUID;

    size_t len = size;
    return sysctl(mib, ARRAY_SIZE(mib), uuid, &len, NULL, 0) != -1;
}

#endif

int device_uuid(char* uuid, size_t size) {
    if (size == 0 || !uuid) {
        return 0;
    }

    int retval = 0;
    if (dmidecode_available() && dmidecode_uuid(uuid, size)) {
        retval = 1;
        goto __exit;
    }

#if defined(__FreeBSD__)
    if (!retval && systctl_uuid(uuid, size)) {
        retval = 1;
        goto __exit;
    }
#endif

__exit:
    if (retval) {
        uuid[size - 1] = '\0';
    }

    return retval;
}