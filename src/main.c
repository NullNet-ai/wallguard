#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "platform/ident.h"
#include "platform/device.h"

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    const platform pl = ident();

    const char *pl_name = platform_name(pl);
    printf("Identified platform:\t%s\n", pl_name);

    char uuid[128];
    memset(uuid, 0, sizeof(uuid));

    if (device_uuid(uuid, sizeof(uuid))) {
        printf("Device UUID: %s\n", uuid);
    } else {
        printf("Failed to obtain device UUID\n");
    }

    return EXIT_SUCCESS;
}