#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "platform/revision.h"
#include "utils/file_utils.h"
#include "utils/url.h"
#include "network/file_transfer.h"
#include "server_api/request_registration.h"

int test_run(const char* url, boolean_t dev) {
    platform_info* info;

    if (!dev) {
        info = get_platform_info();
    } else {
        static platform_info dummy;
        dummy.model   = "Test";
        dummy.version = "1.0.0";
        dummy.type    = -1;

        void* dummy_uuid = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
        memcpy(dummy.uuid, dummy_uuid, 37);

        info = &dummy;
    }

    if (info == NULL) {
        printf("Failed to obtain platfrom info, aborting ...\n");
        return EXIT_FAILURE;
    }

    printf("Platform:\nModel: %s\nVersion: %s\nUUID: %s\n\n", info->model, info->version, info->uuid);

    if (!request_registration(url, info)) {
        printf("Regsitration request to the central server failed, aborting ...\n");

        if (!dev) {
            release_platform_info(info);
        }

        return EXIT_FAILURE;
    } else {
        printf("Registration successful.\n");
    }

    // const char* cfg = "/conf/config.xml";

    // file_monitor mnt;
    // if (!file_monitor_init(&mnt, cfg)) {
    //     printf("Failed to initialize file monitor, verify file exists %s\n", cfg);

    //     if (!dev) {
    //         release_platform_info(info);
    //     }

    //     return EXIT_FAILURE;
    // }

    // printf("Start monitoring ...\n");
    // for (;;) {
    //     sleep(1);

    //     if (!file_monitor_check(&mnt)) {
    //         continue;
    //     }

    //     printf("%s has been changed, uploading to server\n", cfg);
    //     boolean_t ur = upload(hostname, port, "/upload", cfg, tls);

    //     if (ur) {
    //         printf("Upload successful!\n");
    //     } else {
    //         printf("Upload failed!\n");
    //     }
    // }

    return EXIT_SUCCESS;
}

int main(int argc, char** argv) {
    (void)argc;
    (void)argv;

    if (argc < 2) {
        printf("Not enough arguments...\n");
        return EXIT_FAILURE;
    }

    return test_run(argv[1], WM_FALSE);
}