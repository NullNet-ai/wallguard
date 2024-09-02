#include <stdio.h>

static size_t clamp(size_t value, size_t min, size_t max) {
    if (value < min) {
        return min;
    } else if (value > max) {
        return max;
    } else {
        return value;
    }
}

#if defined(__FreeBSD__)
#include <sys/types.h>
#include <sys/sysctl.h>

static boolean_t systctl_uuid(char* uuid, size_t size) {
    int mib[2];

    mib[0] = CTL_KERN;
    mib[1] = KERN_HOSTUUID;

    size_t    len    = size;
    boolean_t result = sysctl(mib, 2, uuid, &len, NULL, 0) != -1;

    uuid[clamp(len, 0, size - 1)] = '\0';
    return result;
}
#endif

int main () {
    char buffer[64] = {0};

    #if defined(__FreeBSD__)
    systctl_uuid(buffer, sizeof(buffer));
    #endif

    printf("UUID: %s\n", buffer);
    return 0;
}