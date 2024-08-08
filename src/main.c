#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "platform/revision.h"
#include "utils/file_utils.h"
#include "utils/url.h"
#include "network/file_transfer.h"

int test_run(const char* url, boolean_t dev) {
    if (!dev) {
        platform_type platform = ident();
        if (platform == PLATFORM_UNSUPPORTED) {
            printf("Unsopported platform, aborting ...\n");
            return EXIT_FAILURE;
        }

        const char* pname = platform_name(platform);
        printf("Platform has been identified as %s\n", pname);
    }

    const char* cfg = "/conf/config.xml";

    file_monitor mnt;
    if (!file_monitor_init(&mnt, cfg)) {
        printf("Failed to initialize file monitor, verify file exists %s\n", cfg);
        return EXIT_FAILURE;
    }

    boolean_t tls  = WM_FALSE;
    int       port = 0;
    char      hostname[256];

    memset(hostname, 0, sizeof(hostname));
    if (!parse_url(url, hostname, &port, &tls)) {
        printf("Failed to parse the server URL ...\n");
        return EXIT_FAILURE;
    }

    printf("Parsed URL:\nHostname: %s\nPort: %d\nUse TLS: %s\n", hostname, port, tls ? "True" : "False");

    printf("Start monitoring ...\n");
    for (;;) {
        sleep(1);

        if (!file_monitor_check(&mnt)) {
            continue;
        }

        printf("%s has been changed, uploading to server\n", cfg);
        boolean_t ur = upload(hostname, port, "/upload", cfg, tls);

        if (ur) {
            printf("Upload successful!\n");
        } else {
            printf("Upload failed!\n");
        }
    }

    return EXIT_SUCCESS;
}

int main(int argc, char** argv) {
    (void)argc;
    (void)argv;

    if (argc < 2) {
        printf("Not enough arguments...\n");
        return EXIT_FAILURE;
    }

    return test_run(argv[1], WM_TRUE);
}