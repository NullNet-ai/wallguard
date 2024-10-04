#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <dirent.h>
#include <stdarg.h>
#include <sys/stat.h>
#include <net/if.h>

#include "logger/logger.h"
#include "platform/bootstrap.h"
#include "utils/file_utils.h"

#include "server_requests.h"
#include "cli_args.h"

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

    WALLMON_LOG_INFO("%10s : %s\n%10s : %s\n%10s : %s\n", "Model", info->model, "Version", info->version, "UUID",
                     info->uuid);

    const char*  cfg = "/conf/config.xml";
    file_monitor mnt;
    if (!file_monitor_init(&mnt, cfg)) {
        WALLMON_LOG_ERROR("Failed to initialize configuration monitor");
        release_platform_info(info);
        return EXIT_FAILURE;
    }

    WALLMON_LOG_INFO("Successfully initialized configuration monitor");

    if (!system_locked() || !validate_lock(info)) {
        WALLMON_LOG_INFO("No lock file found or lock file is invalid, considering this as the first launch.");

        if (!wallmom_registration(info)) {
            WALLMON_LOG_ERROR("Registration failed, aborting ...");
            release_platform_info(info);
            return EXIT_FAILURE;
        }

        WALLMON_LOG_INFO("Registration successfull");

        if (!lock_system(info)) {
            WALLMON_LOG_ERROR("Failed wto write the lock file, aborting ...");
            release_platform_info(info);
            return EXIT_FAILURE;
        }

        WALLMON_LOG_INFO("Successfully written the lock file");
    } else {
        WALLMON_LOG_INFO("Lock file found, proceeding without action");
    }

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
        if ((now - last_heartbeat) >= cli_args.heartbeat_period) {
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
    logger_cleanup();
    release_platform_info(info);
    return EXIT_SUCCESS;
}

static void validate_interface() {
    if (!cli_args.interface) {
        return;
    }

    struct if_nameindex* if_index = if_nameindex();

    if (if_index == NULL) {
        WALLMON_LOG_WARN("Failed to obtain list of interfaces, setting interface to default");
        cli_args.interface = NULL;
        return;
    }

    boolean_t found = WM_FALSE;

    for (struct if_nameindex* iface = if_index; iface->if_index != 0 || iface->if_name != NULL; ++iface) {
        if (strcmp(iface->if_name, cli_args.interface) == 0) {
            found = WM_TRUE;
            break;
        }
    }

    if_freenameindex(if_index);

    if (!found) {
        WALLMON_LOG_WARN("%s interface has not been found, setting to default.", cli_args.interface);
        cli_args.interface = NULL;
    }
}

int main(int argc, char** argv) {
    parse_cli_arguments(argc, argv);
    logger_init(argv[0], LOGGER_TYPE_CONSOLE | LOGGER_TYPE_FILE | LOGGER_TYPE_SYSLOG, LOG_SEVERITY_INFO);
    validate_interface();
    return wallmon_main();
}
