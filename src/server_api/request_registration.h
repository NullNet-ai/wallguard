#ifndef SERVER_API_REQUEST_REGISTRATION_H
#define SERVER_API_REQUEST_REGISTRATION_H

#include <utils/common.h>
#include <platform/ident.h>

/**
 * @brief Sends a request to the central server to register the current platform.
 *
 * @param server_url The URL of the server where the registration request is sent.
 * @param info A pointer to a `platform_info` structure containing details about the current platform.
 * @return `WM_TRUE` if the registration is successful, `WM_FALSE` if the registration fails.
 *
 * @note The `platform_info` structure should be properly populated with the necessary platform details before calling this function.
 */
boolean_t request_registration(const char* server_url, platform_info* info);
#endif
