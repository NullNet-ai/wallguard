#include <platform/device.h>
#include <utils/file_utils.h>
#include <utils/common.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static size_t clamp(size_t value, size_t min, size_t max) {
    if (value < min) {
        return min;
    } else if (value > max) {
        return max;
    } else {
        return value;
    }
}

static boolean_t dmidecode_available() {
#if defined(__FreeBSD__)
    const char* dmidecode_path = "/usr/local/sbin/dmidecode";
#else
    const char* dmidecode_path = "/usr/sbin/dmidecode";
#endif
    return file_exists(dmidecode_path);
}

static boolean_t dmidecode_uuid(char* uuid, size_t size) {
    boolean_t retval = WM_TRUE;
    FILE*     pipe   = popen("dmidecode -s system-uuid", "r");

    if (!pipe) {
        retval = WM_FALSE;
        goto __exit;
    }

    if (fgets(uuid, size, pipe) == NULL) {
        retval = WM_FALSE;
        goto __exit;
    }

    size_t len = strcspn(uuid, "\n");

    uuid[clamp(len, 0, size - 1)] = '\0';

__exit:
    if (pipe) {
        pclose(pipe);
    }
    return retval;
}

#if defined(__FreeBSD__)
#include <sys/types.h>
#include <sys/sysctl.h>

static boolean_t systctl_uuid(char* uuid, size_t size) {
    int mib[2];

    mib[0] = CTL_KERN;
    mib[1] = KERN_HOSTUUID;

    size_t    len    = size;
    boolean_t result = sysctl(mib, ARRAY_SIZE(mib), uuid, &len, NULL, 0) != -1;

    uuid[clamp(len, 0, size - 1)] = '\0';
    return result;
}

#endif

boolean_t device_uuid(char* uuid, size_t size) {
    if (size == 0 || !uuid) {
        return WM_FALSE;
    }

    if (dmidecode_available() && dmidecode_uuid(uuid, size)) {
        return WM_TRUE;
    }

#if defined(__FreeBSD__)
    if (systctl_uuid(uuid, size)) {
        return WM_TRUE;
    }
#endif

    return WM_FALSE;
}

// @TODO: Read `/sys/class/dmi/id/product_uuid` file for the system UUID on Linux
