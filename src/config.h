#ifndef _CONFIG_H_
#define _CONFIG_H_

#include <utils/common.h>

/**
 * @brief Initializes the configuration system by loading the configuration file.
 *
 * @param filename The path to the configuration file to be loaded.
 */
void cfg_init(const char* filename);

/**
 * @brief Deinitializes the configuration system, releasing any allocated resources.
 */
void cfg_deinit(void);

/**
 * @brief Validates the current configuration for completeness.
 */
void cfg_validate(void);

/**
 * @brief Retrieves the server URL from the configuration.
 *
 * @return A string representing the server URL.
 */
const char* cfg_get_server_url();

/**
 * @brief Retrieves the instance name from the configuration.
 *
 * @return A string representing the instance name.
 */
const char* cfg_get_instance_name();

/**
 * @brief Retrieves the API key from the configuration.
 *
 * @return A string representing the API key.
 */
const char* cfg_get_api_key();

/**
 * @brief Retrieves the access token from the configuration.
 *
 * @return A string representing the access token.
 */
const char* cfg_get_access_token();

/**
 * @brief Retrieves the network interface name from the configuration.
 *
 * @return A string representing the network interface name.
 */
const char* cfg_get_netwrok_interface();

/**
 * @brief Retrieves the system UUID from the configuration.
 *
 * @return A string representing the system UUID.
 */
const char* cfg_get_system_uuid();

/**
 * @brief Retrieves the platform identifier from the configuration.
 *
 * @return A string representing the platform identifier.
 */
const char* cfg_get_platform();

/**
 * @brief Retrieves the heartbeat interval in seconds from the configuration.
 *
 * @return An integer representing the heartbeat interval in seconds.
 */
int cfg_get_heartbeat_interval();

/**
 * @brief Retrieves the monitor URL from the configuration.
 *
 * @return A string representing the monitor URL.
 */
const char* cfg_get_monitor_url();

/**
 * @brief Retrieves the configuration value that determines whether server traffic
 *        should be filtered out from packet capture or processing.
 *
 * @return `WM_TRUE` If server traffic should be filtered out, `WM_FALSE` otherwise.
 */
boolean_t cfg_get_filter_out_server_traffic();

#endif
