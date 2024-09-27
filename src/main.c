#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <dirent.h>
#include <stdarg.h>
#include <sys/stat.h>

#include "logger/logger.h"
#include "platform/bootstrap.h"
#include "utils/file_utils.h"

#include "server_requests.h"

const char* start_message =
    "\n"
    "               | | | \n"
    " __      ____ _| | |_ __ ___   ___  _ __  \n"
    " \\ \\ /\\ / / _` | | | '_ ` _ \\ / _ \\| '_ \\ \n"
    "  \\ V  V / (_| | | | | | | | | (_) | | | |\n"
    "   \\_/\\_/ \\__,_|_|_|_| |_| |_|\\___/|_| |_|\n";

// @TODO: Refine the criteria
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

    if (!system_locked() || !validate_lock(info)) {
        WLOG_INFO("No lock file found or lock file is invalid, considering this as the first launch.");

        if (!wallmom_registration(url, info)) {
            WLOG_ERROR("Registration failed, aborting ...");
            release_platform_info(info);
            return EXIT_FAILURE;
        }

        WLOG_INFO("Registration successfull");

        if (!lock_system(info)) {
            WLOG_ERROR("Failed wto write the lock file, aborting ...");
            release_platform_info(info);
            return EXIT_FAILURE;
        }

        WLOG_INFO("Successfully written the lock file");
    } else {
        WLOG_INFO("Lock file found, proceeding without action");
    }

    boolean_t current_state  = is_system_dirty();
    time_t    last_heartbeat = 0;

    // Send to the server the most recent configuration
    if (!wallmon_uploadcfg(url, cfg, info, !is_system_dirty())) {
        WLOG_ERROR("Failed to upload the intial configuration to the server, aborting ... ");
        release_platform_info(info);
        return EXIT_FAILURE;
    } else {
        WLOG_INFO("Successfully uploaded the initial configuration to the server.");
    }

    for (;;) {
        // Send heartbeat every 60 seconds
        time_t now = time(NULL);
        if ((now - last_heartbeat) >= 60) {
            last_heartbeat = now;

            if (wallmon_heartbeat(url, info)) {
                WLOG_INFO("Heartbeat sent");
            } else {
                WLOG_ERROR("Heartbeat failed");
            }
        }

        boolean_t state = is_system_dirty();

        if (file_monitor_check(&mnt) == 1) {
            if (wallmon_uploadcfg(url, cfg, info, !state)) {
                WLOG_INFO("Configuration uploaded successfully");
            } else {
                WLOG_ERROR("Failed to upload configuration");
            }
        }

        if (state ^ current_state) {
            current_state = state;

            if (current_state) {
                // System has just became dirty
                continue;
            }
            WLOG_INFO("Configraution reload detected");

            if (wallmon_uploadcfg(url, cfg, info, !state)) {
                WLOG_INFO("Configuration uploaded successfully");
            } else {
                WLOG_ERROR("Failed to upload configuration");
            }
        }

        sleep(1);
    }

    // @TODO: Unreachable code
    release_platform_info(info);
    return EXIT_SUCCESS;
}

static void initialize_logger(void) {
    WLOG_SET_TYPE_FLAG(LOGGER_TYPE_CONSOLE);
    WLOG_SET_TYPE_FLAG(LOGGER_TYPE_FILE);
    WLOG_SET_LOG_LEVEL(LOG_SEVERITY_INFO);
}

int main(int argc, char** argv) {
    if (argc < 2) {
        printf("Not enought arguments, aborting\n");
        return EXIT_FAILURE;
    }

    printf("%s\n", start_message);
    initialize_logger();
    return wallmon_main(argv[1]);
}
