#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "platform/revision.h"
#include "utils/file_utils.h"
#include "network/file_transfer.h"

int test_run(boolean_t dev) {
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

    printf("Start monitoring ...\n");
    for (;;) {
        sleep(1);

        if (!file_monitor_check(&mnt)) {
            continue;
        }

        printf("%s has been changed, uploading to server\n", cfg);

        boolean_t ur = upload("192.168.2.19", 3000, "/upload", cfg, WM_FALSE);
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

    return test_run(WM_FALSE);
}