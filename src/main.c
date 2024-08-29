#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <dirent.h>

#include "utils/file_utils.h"
#include "utils/url.h"

#include "server_api/request_registration.h"
#include "server_api/upload_configuration.h"
#include "server_api/notify_configuration_reload.h"
#include "server_api/heartbeat.h"

static boolean_t is_system_dirty() {
    DIR* directory = opendir("/var/run/");
    if (!directory) {
        return WM_FALSE;
    }

    int8_t retval = WM_FALSE;

    struct dirent* info;
    while ((info = readdir(directory)) != NULL) {
        const char* ext = extension(info->d_name);
        if (ext && strcmp(ext, "dirty") == 0) {
            retval = WM_TRUE;
            break;
        }
    }

    closedir(directory);
    return retval;
}

int wallmon_main(const char* url, boolean_t dev) {
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

    const char* cfg = "/conf/config.xml";

    file_monitor mnt;
    if (!file_monitor_init(&mnt, cfg)) {
        printf("Failed to initialize file monitor, verify file exists %s\n", cfg);

        if (!dev) {
            release_platform_info(info);
        }

        return EXIT_FAILURE;
    }

    printf("Start monitoring ...\n");

    boolean_t current_state = WM_FALSE;

    time_t last_heartbeat = 0;

    for (;;) {
        // Send heartbeat every 60 seconds
        time_t now = time(NULL);
        if ((now - last_heartbeat) >= 60) {
            last_heartbeat = now;

            printf("Sending heartbeat -> ");
            if (heartbeat_request(url, info)) {
                printf("Success\n");
            } else {
                printf("Failure\n");
            }
        }

        if (file_monitor_check(&mnt) == 1) {
            printf("%s has been changed, uploading to server -> ", cfg);

            if (upload_configuration(url, cfg, info)) {
                printf("Success\n");
            } else {
                printf("Failure\n");
            }
        }

        boolean_t state = is_system_dirty();

        if (state ^ current_state) {
            current_state = state;

            // if dirty
            if (current_state) {
                continue;
            }

            printf("Configuration has been reloaded, notifying the server -> ");

            if (notify_configuration_reload(url, info)) {
                printf("Success\n");
            } else {
                printf("Failure\n");
            }
        }

        sleep(1);
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

    return wallmon_main(argv[1], WM_TRUE);
}