#include "config.h"

#include <libconfig.h>
#include <stdlib.h>
#include <stdio.h>
#include <logger/logger.h>
#include <utils/common.h>

static config_t configuration;

void cfg_init(const char* filename) {
    config_init(&configuration);

    if (!config_read_file(&configuration, filename)) {
        WALLMON_LOG_ERROR("Error reading configuration file %s at line %d: %s", filename,
                          config_error_line(&configuration), config_error_text(&configuration));

        config_destroy(&configuration);
        exit(EXIT_FAILURE);
    }
}

void cfg_deinit(void) { config_destroy(&configuration); }

void cfg_validate(void) {
    boolean_t valid = WM_TRUE;

    if (!cfg_get_server_url()) {
        WALLMON_LOG_ERROR("'server_url' is absent in the configuration file");
        valid = WM_FALSE;
    }

    if (!cfg_get_instance_name()) {
        WALLMON_LOG_ERROR("'instance_name' is missing in the configuration file");
        valid = WM_FALSE;
    }

    if (!cfg_get_api_key()) {
        WALLMON_LOG_ERROR("'api_key' is missing in the configuration file");
        valid = WM_FALSE;
    }

    if (!cfg_get_access_token()) {
        WALLMON_LOG_ERROR("'access_token' is missing in the configuration file");
        valid = WM_FALSE;
    }

    if (!cfg_get_system_uuid()) {
        WALLMON_LOG_ERROR("'system_uuid' is missing in the configuration file");
        valid = WM_FALSE;
    }

    if (!cfg_get_platform()) {
        WALLMON_LOG_ERROR("'platform' is missing in the configuration file");
        valid = WM_FALSE;
    }

    if (!cfg_get_monitor_url()) {
        WALLMON_LOG_ERROR("'monitor_url' is missing in the configuration file");
        valid = WM_FALSE;
    }

    if (!valid) {
        cfg_deinit();
        exit(EXIT_FAILURE);
    }
}

const char* cfg_get_server_url() {
    const char* server_url = NULL;

    if (config_lookup_string(&configuration, "server_url", &server_url)) {
        return server_url;
    }

    return NULL;
}

const char* cfg_get_instance_name() {
    const char* instance_name = NULL;

    if (config_lookup_string(&configuration, "instance_name", &instance_name)) {
        return instance_name;
    }

    return NULL;
}

const char* cfg_get_api_key() {
    const char* api_key = NULL;

    if (config_lookup_string(&configuration, "api_key", &api_key)) {
        return api_key;
    }

    return NULL;
}

const char* cfg_get_access_token() {
    const char* access_token = NULL;

    if (config_lookup_string(&configuration, "access_token", &access_token)) {
        return access_token;
    }

    return NULL;
}

const char* cfg_get_netwrok_interface() {
    const char* network_interface = NULL;

    if (config_lookup_string(&configuration, "network_interface", &network_interface)) {
        return network_interface;
    }

    return NULL;
}

const char* cfg_get_system_uuid() {
    const char* system_uuid = NULL;

    if (config_lookup_string(&configuration, "system_uuid", &system_uuid)) {
        return system_uuid;
    }

    return NULL;
}

const char* cfg_get_platform() {
    const char* platform = NULL;

    if (config_lookup_string(&configuration, "platform", &platform)) {
        return platform;
    }

    return NULL;
}

int cfg_get_heartbeat_interval() {
    int heartbeat_interval = 0;

    if (config_lookup_int(&configuration, "heartbeat_interval", &heartbeat_interval)) {
        return heartbeat_interval;
    }

    return 60;
}

const char* cfg_get_monitor_url() {
    const char* monitor_url = NULL;

    if (config_lookup_string(&configuration, "monitor_url", &monitor_url)) {
        return monitor_url;
    }

    return NULL;
}

boolean_t cfg_get_filter_out_server_traffic() {
    int value = 0;
    if (config_lookup_bool(&configuration, "ignore_server_packets", &value)) {
        // Not nessesary, but it is here for consistency
        return value == CONFIG_TRUE ? WM_TRUE : WM_FALSE;
    }

    return WM_FALSE;
}
