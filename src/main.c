#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <dirent.h>
#include <stdarg.h>

#include "utils/file_utils.h"
#include "utils/url.h"

#include "server_api/request_registration.h"
#include "server_api/upload_configuration.h"
#include "server_api/notify_configuration_reload.h"
#include "server_api/heartbeat.h"

#define RED "\x1B[31m"
#define GRN "\x1B[32m"
#define YEL "\x1B[33m"
#define RESET "\x1B[0m"

#define LOG_STEP(text) printf(text " -> ")
#define LOG_STEP_SUCCESS() printf(GRN "Success" RESET "\n")
#define LOG_STEP_FAILURE() printf(RED "Failure" RESET "\n")

static boolean_t check_if_first_launch() {
    const char* lockfile = "/var/lock/wallmon.lock";
    if (file_exists(lockfile)) {
        return WM_TRUE;
    }

    LOG_STEP("First launch, writing lock file");

    FILE* file = fopen(lockfile, "w");
    if (file) {
        LOG_STEP_SUCCESS();
        fclose(file);
    } else {
        LOG_STEP_FAILURE();
    }

    return WM_TRUE;
}

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

int wallmon_main(const char* url) {
    LOG_STEP("Obtaining platform info");

    platform_info* info = get_platform_info();
    if (info == NULL) {
        LOG_STEP_FAILURE();
        return EXIT_FAILURE;
    } else {
        LOG_STEP_SUCCESS();
    }

    printf("Model: " YEL " %s " RESET "\nVersion:" YEL " %s " RESET " \nUUID: " YEL " %s " RESET " \n", info->model,
           info->version, info->uuid);

    LOG_STEP("Initializing configuration monitor");

    const char*  cfg = "/conf/config.xml";
    file_monitor mnt;
    if (!file_monitor_init(&mnt, cfg)) {
        LOG_STEP_FAILURE();
        release_platform_info(info);
        return EXIT_FAILURE;
    } else {
        LOG_STEP_SUCCESS();
    }

    LOG_STEP("Requesting registation");
    if (!request_registration(url, info)) {
        LOG_STEP_FAILURE();
        release_platform_info(info);
        return EXIT_FAILURE;
    } else {
        LOG_STEP_SUCCESS();
    }

    if (check_if_first_launch()) {
        LOG_STEP("First launch, uploading configuration");
        if (upload_configuration(url, cfg, info)) {
            LOG_STEP_SUCCESS();
        } else {
            LOG_STEP_FAILURE();
        }
    }

    boolean_t current_state  = WM_FALSE;
    time_t    last_heartbeat = 0;

    for (;;) {
        // Send heartbeat every 60 seconds
        time_t now = time(NULL);
        if ((now - last_heartbeat) >= 60) {
            last_heartbeat = now;

            LOG_STEP("Sending heartbeat");
            if (heartbeat_request(url, info)) {
                LOG_STEP_SUCCESS();
            } else {
                LOG_STEP_FAILURE();
            }
        }

        if (file_monitor_check(&mnt) == 1) {
            LOG_STEP("Notifying configuration change");
            if (upload_configuration(url, cfg, info)) {
                LOG_STEP_SUCCESS();
            } else {
                LOG_STEP_FAILURE();
            }
        }

        boolean_t state = is_system_dirty();

        if (state ^ current_state) {
            current_state = state;

            // if dirty
            if (current_state) {
                continue;
            }

            LOG_STEP("Notifying configuration reload");
            if (notify_configuration_reload(url, info)) {
                LOG_STEP_SUCCESS();
            } else {
                LOG_STEP_FAILURE();
            }
        }

        sleep(1);
    }

    release_platform_info(info);
    return EXIT_SUCCESS;
}

int main(int argc, char** argv) {
    (void)argc;
    (void)argv;

    if (argc < 2) {
        printf("Not enough arguments...\n");
        return EXIT_FAILURE;
    }

    return wallmon_main(argv[1]);
}
