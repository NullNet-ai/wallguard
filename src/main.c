#include <stdlib.h>
#include <stdio.h>
#include <stddef.h>
#include <string.h>
#include <unistd.h>
#include <dirent.h>
#include <stdarg.h>
#include <sys/stat.h>

#include "logger/logger.h"
#include "utils/file_utils.h"
#include "utils/str.h"
#include "utils/net.h"
#include "server_requests.h"
#include "cli_args.h"
#include "config.h"

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

int wallmon_main() {
    platform_info* info = get_platform_info();
    if (info == NULL) {
        WALLMON_LOG_ERROR("Failed to obtain the platform info");
        return EXIT_FAILURE;
    }

    const char*  cfg = "/conf/config.xml";
    file_monitor mnt;
    if (!file_monitor_init(&mnt, cfg)) {
        WALLMON_LOG_ERROR("Failed to initialize configuration monitor");
        release_platform_info(info);
        return EXIT_FAILURE;
    }

    WALLMON_LOG_INFO("Successfully initialized configuration monitor");

    // if (!system_locked() || !validate_lock(info)) {
    //     WALLMON_LOG_INFO("No lock file found or lock file is invalid, considering this as the first launch.");

    //     if (!wallmom_registration(info)) {
    //         WALLMON_LOG_ERROR("Registration failed, aborting ...");
    //         release_platform_info(info);
    //         return EXIT_FAILURE;
    //     }

    //     WALLMON_LOG_INFO("Registration successfull");

    //     if (!lock_system(info)) {
    //         WALLMON_LOG_ERROR("Failed wto write the lock file, aborting ...");
    //         release_platform_info(info);
    //         return EXIT_FAILURE;
    //     }

    //     WALLMON_LOG_INFO("Successfully written the lock file");
    // } else {
    //     WALLMON_LOG_INFO("Lock file found, proceeding without action");
    // }

    // Send to the server the most recent configuration
    if (!wallmon_uploadcfg(cfg, info, !is_system_dirty())) {
        WALLMON_LOG_ERROR("Failed to upload the intial configuration to the server, aborting ... ");
        release_platform_info(info);
        return EXIT_FAILURE;
    } else {
        WALLMON_LOG_INFO("Successfully uploaded the initial configuration to the server.");
    }

    boolean_t current_state  = is_system_dirty();
    time_t    last_heartbeat = 0;

    for (;;) {
        time_t now = time(NULL);
        if ((now - last_heartbeat) >= cfg_get_heartbeat_interval()) {
            last_heartbeat = now;

            if (wallmon_heartbeat(info)) {
                WALLMON_LOG_INFO("Heartbeat sent");
            } else {
                WALLMON_LOG_ERROR("Heartbeat failed");
            }
        }

        boolean_t state = is_system_dirty();

        if (file_monitor_check(&mnt) == 1) {
            if (wallmon_uploadcfg(cfg, info, !state)) {
                WALLMON_LOG_INFO("Configuration uploaded successfully");
            } else {
                WALLMON_LOG_ERROR("Failed to upload configuration");
            }
        }

        if (state ^ current_state) {
            current_state = state;

            if (current_state) {
                // System has just became dirty
                continue;
            }
            WALLMON_LOG_INFO("Configraution reload detected");

            if (wallmon_uploadcfg(cfg, info, !state)) {
                WALLMON_LOG_INFO("Configuration uploaded successfully");
            } else {
                WALLMON_LOG_ERROR("Failed to upload configuration");
            }
        }

        sleep(1);
    }

    // @TODO: Unreachable code
    release_platform_info(info);
    return EXIT_SUCCESS;
}

static void validate_system_uuid() {
    const char* system_uuid = cfg_get_system_uuid();
    if (!is_valid_uuid(system_uuid)) {
        WALLMON_LOG_ERROR("Provided System ID '%s' is not a valid UUID", system_uuid);
        cfg_deinit();
        logger_cleanup();
        exit(EXIT_FAILURE);
    }
}

static void validate_network_interface() {
    const char* ifname = cfg_get_netwrok_interface();

    if (!ifname) {
        // Not specified, use system's default
        return;
    }

    if (!is_interface_valid(ifname)) {
        WALLMON_LOG_ERROR("Provided network interface '%s' does not exist", ifname);
        cfg_deinit();
        logger_cleanup();
        exit(EXIT_FAILURE);
    }
}

static void validate_platform() {
    const char*   platform      = cfg_get_platform();
    platform_type platform_type = get_platform_type();

    if (platform_type == PLATFORM_UNSUPPORTED) {
        WALLMON_LOG_ERROR("Provided plaform '%s' is not supported", platform);
        cfg_deinit();
        logger_cleanup();
        exit(EXIT_FAILURE);
    }
}

int main(int argc, char** argv) {
    parse_cli_arguments(argc, argv);
    logger_init(argv[0], LOGGER_TYPE_CONSOLE | LOGGER_TYPE_FILE | LOGGER_TYPE_SYSLOG, LOG_SEVERITY_INFO);

    cfg_init(cli_args.config_filename);
    cfg_validate();

    validate_system_uuid();
    validate_network_interface();
    validate_platform();

    // int exit_code = wallmon_main();
    int exit_code = 0;

    cfg_deinit();
    logger_cleanup();
    return exit_code;
}
