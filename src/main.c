#include <stdlib.h>
#include <stdio.h>
#include <stddef.h>
#include <string.h>
#include <unistd.h>
#include <stdarg.h>
#include <sys/stat.h>
#include <signal.h>
#include <pthread.h>

#include "logger/logger.h"
#include "utils/file_utils.h"
#include "utils/str.h"
#include "utils/net.h"
#include "server_requests.h"
#include "cli_args.h"
#include "config.h"
#include "crypto.h"
#include "upload_sequence.h"
#include "net_capture/sniffer.h"
#include "utils/wsignal.h"

static const char backups_directory[] = "/var/backups/wallmon";
static const char configurationfile[] = "/conf/config.xml";

static boolean_t backup_and_upload(const char* session_token, platform_info* info) {
    char backup_filename[256] = {0};
    snprintf(backup_filename, sizeof(backup_filename), "%s/%ld_%s.xml", backups_directory, time(NULL), info->uuid);

    if (!copy_file(configurationfile, backup_filename)) {
        WALLMON_LOG_ERROR("backup_and_upload: Failed to copy %s to %s", configurationfile, backup_filename);
        return WM_FALSE;
    }

    if (!upload_sequence(session_token, backup_filename, info)) {
        WALLMON_LOG_ERROR("backup_and_upload: Upload sequence failed");
        return WM_FALSE;
    }

    return WM_TRUE;
}

static void heartbeat(time_t* last_heartbeat, const char* session_token) {
    time_t diff = time(NULL) - *last_heartbeat;

    if (diff < cfg_get_heartbeat_interval()) {
        return;
    }

    if (!wallmon_heartbeat(session_token)) {
        WALLMON_LOG_ERROR("heartbeat failed");
    }

    *last_heartbeat = time(NULL);
}

static void* monitoring_routing(void* arg) {
    sniffer_t* sniffer = (sniffer_t*)arg;
    sniffer_mainloop(sniffer);
    return NULL;
}

static int main_loop(platform_info* info) {
    if (!make_directory(backups_directory)) {
        WALLMON_LOG_ERROR("main_loop: Could not create directory %s", backups_directory);
        return EXIT_FAILURE;
    }

    file_monitor mnt;
    if (!file_monitor_init(&mnt, configurationfile)) {
        WALLMON_LOG_ERROR("main_loop: Failed to initialize configuration monitor");
        return EXIT_FAILURE;
    }

    char* session_token = NULL;
    if (!wallmon_authenticate(&session_token)) {
        WALLMON_LOG_ERROR("main_loop: Authentication failed");
        return EXIT_FAILURE;
    }

    if (!wallmon_setup(session_token, info)) {
        WALLMON_LOG_ERROR("main_loop: Setup failed");
        return EXIT_FAILURE;
    }

    if (!backup_and_upload(session_token, info)) {
        WALLMON_LOG_ERROR("main_loop: Initial configuration upload failed");
        W_FREE(session_token);
        return EXIT_FAILURE;
    }

    char* monitor_public_key = NULL;
    if(!wallmon_fetch_monitor_key(session_token, &monitor_public_key)) {
        WALLMON_LOG_ERROR("main_loop: Failed to fetch monitor key");
        W_FREE(session_token);
        return EXIT_FAILURE;
    }

    sniffer_t* sniffer = sniffer_initialize(monitor_public_key);
    if (!sniffer) {
        WALLMON_LOG_ERROR("main_loop: Failed to initialize traffic monitoring");
        W_FREE(session_token);
        W_FREE(monitor_public_key);
        return EXIT_FAILURE;
    }

    pthread_t sniffer_thread;
    if(pthread_create(&sniffer_thread, NULL, monitoring_routing, sniffer) != 0) {
        WALLMON_LOG_ERROR("main_loop: Failed to spawn traffic monitoring thread");
        W_FREE(session_token);
        W_FREE(monitor_public_key);
        sniffer_finalize(sniffer);
        return EXIT_FAILURE;
    }

    WALLMON_LOG_INFO("main_loop: Start");

    time_t last_heartbeat = 0;
    while (wallmon_is_running()) {
        heartbeat(&last_heartbeat, session_token);

        boolean_t is_dirty = info->dirty;
        update_platform_info(info);

        /// Transition from dirty to clean or file monitor detected changes
        boolean_t should_upload = (is_dirty ^ info->dirty && !info->dirty) || file_monitor_check(&mnt);

        if (should_upload) {
            if (upload_sequence(session_token, configurationfile, info)) {
                WALLMON_LOG_INFO("main_loop: Successfully uploaded %s to the server", configurationfile);
            } else {
                WALLMON_LOG_ERROR("main_loop: Failed to upload %s to server", configurationfile);
            }
        }

        if (wallmon_is_running()) {
            sleep(1);
        }
    }

    pthread_join(sniffer_thread, NULL);

    W_FREE(session_token);
    W_FREE(monitor_public_key);
    sniffer_finalize(sniffer);
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

    wallmon_setup_signal_handler();

    int exitcode = 0;
    

    platform_info* info = get_platform_info();
    if (info) {
        exitcode = main_loop(info);
        release_platform_info(info);
    } else {
        WALLMON_LOG_ERROR("Failed to obtain platform info, aborting ...");
    }


    cfg_deinit();
    logger_cleanup();
    return exitcode;
}
