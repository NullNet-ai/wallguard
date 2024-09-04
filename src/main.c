#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <dirent.h>
#include <stdarg.h>
#include <sys/stat.h>

#include "logger/logger.h"

#include "utils/file_utils.h"
#include "utils/url.h"

#include "server_api/request_registration.h"
#include "server_api/upload_configuration.h"
#include "server_api/notify_configuration_reload.h"
#include "server_api/heartbeat.h"

const char* start_message =
    "               | | | \n"
    " __      ____ _| | |_ __ ___   ___  _ __  \n"
    " \\ \\ /\\ / / _` | | | '_ ` _ \\ / _ \\| '_ \\ \n"
    "  \\ V  V / (_| | | | | | | | | (_) | | | |\n"
    "   \\_/\\_/ \\__,_|_|_|_| |_| |_|\\___/|_| |_|\n";

static boolean_t check_if_first_launch() {
    const char* lockfile = "/var/lock/wallmon.lock";

    if (file_exists(lockfile)) {
        return WM_TRUE;
    }

    if (!directory_exists("/var/lock")) {
        mkdir("/var/lock", 0777);
    }

    FILE* file = fopen(lockfile, "w");
    if (file) {
        WLOG_INFO("First launch detected, wrote lockfile %s", lockfile);
        fclose(file);
    } else {
        WLOG_INFO("Failed to write lockfile to %s", lockfile);
    }

    return WM_TRUE;
}

static void initial_configuration_upload(const char* url, const char* cfg, platform_info* info) {
    if (!check_if_first_launch()) {
        return;
    }

    if (upload_configuration(url, cfg, info) && notify_configuration_reload(url, info)) {
        WLOG_INFO("Successfully uploaded initial configuration to the server");
    } else {
        WLOG_ERROR("Failed to upload  initial configuration to the server");
    }

    // @TODO: Currently done in 2 steps: Upload and Confirm
    // Probably should have its own endpoint and done in 1 API call
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
    platform_info* info = get_platform_info();
    if (info == NULL) {
        WLOG_ERROR("Failed to obtain the platform info");
        return EXIT_FAILURE;
    }

    WLOG_INFO("%10s : %s\n%10s : %s\n%10s : %s\n", "Model", info->model, "Version", info->version, "UUID", info->uuid);

    const char*  cfg = "/conf/config.xml";
    file_monitor mnt;
    if (!file_monitor_init(&mnt, cfg)) {
        WLOG_ERROR("Failed to initialize configuration monitor");
        release_platform_info(info);
        return EXIT_FAILURE;
    }

    WLOG_INFO("Successfully initialized configuration monitor");

    if (!request_registration(url, info)) {
        WLOG_ERROR("Registration failed.");
        release_platform_info(info);
        return EXIT_FAILURE;
    }

    WLOG_INFO("Registration successfull");

    initial_configuration_upload(url, cfg, info);

    boolean_t current_state  = WM_FALSE;
    time_t    last_heartbeat = 0;

    for (;;) {
        // Send heartbeat every 60 seconds
        time_t now = time(NULL);
        if ((now - last_heartbeat) >= 60) {
            last_heartbeat = now;

            if (heartbeat_request(url, info)) {
                WLOG_INFO("Heartbeat sent");
            } else {
                WLOG_ERROR("Heartbeat failed");
            }
        }

        if (file_monitor_check(&mnt) == 1) {
            if (upload_configuration(url, cfg, info)) {
                WLOG_INFO("Configuration uploaded successfully");
            } else {
                WLOG_ERROR("Failed to upload configuration");
            }
        }

        boolean_t state = is_system_dirty();

        if (state ^ current_state) {
            current_state = state;

            // if dirty
            if (current_state) {
                continue;
            }

            if (notify_configuration_reload(url, info)) {
                WLOG_INFO("Successfully notified about the configuration reload");

            } else {
                WLOG_ERROR("Reload notification failed");
            }
        }

        sleep(1);
    }

    release_platform_info(info);
    return EXIT_SUCCESS;
}

#include <logger/logger.h>

static void initialize_logger(void) {
    WLOG_SET_TYPE_FLAG(LOGGER_TYPE_CONSOLE);
    WLOG_SET_TYPE_FLAG(LOGGER_TYPE_FILE);

    WLOG_SET_LOG_LEVEL(LOG_SEVERITY_INFO);
}

int main(int argc, char** argv) {
    (void)argc;
    (void)argv;

    initialize_logger();
    WLOG_INFO(start_message);

    if (argc < 2) {
        WLOG_ERROR("Not enought arguments, aborting");
        return EXIT_FAILURE;
    }

    return wallmon_main(argv[1]);
}
