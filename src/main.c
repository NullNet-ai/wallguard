#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "platform/ident.h"
#include "platform/device.h"
#include "utils/file_utils.h"
#include "network/request.h"

void r1() {
    char request[] =
        "GET / HTTP/1.1\r\n"
        "Host: example.com\r\n"
        "Connection: close\r\n"
        "\r\n";

    https_request("example.com", 443, request, sizeof(request));
}

void r2() {
    char request[] =
        "GET / HTTP/1.1\r\n"
        "Host: 192.168.2.19:8000\r\n"
        "Connection: close\r\n"
        "\r\n";

    http_request("192.168.2.19", 8000, request, sizeof(request));
}

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

    const char *filename = "./_spikes/sampleConfig.xml";
    printf("Stat monitoring file: %s\n", filename);

    if (!file_exists(filename)) {
        printf("File does not exist\n");
        return EXIT_FAILURE;
    }

    file_monitor fmonitor;
    file_monitor_init(&fmonitor, filename);

    r1();
    r2();

    for (;;) {
        int result = file_monitor_check(&fmonitor);
        if (result == -1) {
            printf("Something happend to the file, aboring ... ");
            return EXIT_FAILURE;
        }

        if (result == 1) {
            printf("File has been changed!\n");
            fflush(stdout);
        }

        sleep(1);
    }

    return EXIT_SUCCESS;
}