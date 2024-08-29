#ifndef SERVER_API_NOTIFY_CONFIGURATION_RELOAD_H
#define SERVER_API_NOTIFY_CONFIGURATION_RELOAD_H

#include <utils/common.h>
#include <platform/ident.h>

/**
 * @brief Sends a request to the central server to notify that the configuration.has been reloaded.
 *
 * @param server_url The URL of the server where the registration request is sent.
 * @return `WM_TRUE` if the notification is successful, `WM_FALSE` otherwise.
 *
 * @note The `platform_info` structure should be properly populated with the necessary platform details before calling
 * this function.
 */
boolean_t notify_configuration_reload(const char* server_url, platform_info* info);

#endif
