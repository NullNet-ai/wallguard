#ifndef _SERVER_REQUESTS_H_
#define _SERVER_REQUESTS_H_

#include <utils/common.h>
#include <platform/ident.h>

/**
 * @brief Sends a request to the central server to register the current platform.
 *
 * @param info A pointer to a `platform_info` structure containing details about the current platform.
 * @return `WM_TRUE` if the registration is successful, `WM_FALSE` if the registration fails.
 */
boolean_t wallmom_registration(platform_info* info);

/**
 * @brief Sends a heartbeat signal to the central server to indicate that the system is still active.
 *
 * @param info A pointer to a `platform_info` structure containing details about the current platform.
 * @return `WM_TRUE` if the request is successful, `WM_FALSE` otherwise.
 */
boolean_t wallmon_heartbeat(platform_info* info);

/**
 * @brief Uploads a configuration file to a specified server.
 *
 *
 * @param path       A string representing the full path to the configuration
 *                   file on the local filesystem that needs to be uploaded.
 * @param info       A pointer to a platform_info structure that contains
 *                   platform-specific data that will be included in the
 *                   upload request.
 * @param applied    A value indicating if the configuration file
 *                   is applied or being pending.
 *
 * @return The function returns `WM_TRUE` if the upload is successful, and `WM_FALSE` if it fails.
 */
boolean_t wallmon_uploadcfg(const char* path, platform_info* info, boolean_t applied);

#endif
