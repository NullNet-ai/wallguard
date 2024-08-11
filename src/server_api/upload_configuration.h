#ifndef SERVER_API_UPLOAD_CONFIGURATION_H
#define SERVER_API_UPLOAD_CONFIGURATION_H

#include <utils/common.h>
#include <platform/ident.h>

/**
 * @brief Uploads a configuration file to a specified server.
 *
 *
 * @param server_url A string representing the URL of the server to which
 *                   the configuration file will be uploaded.
 * @param path       A string representing the full path to the configuration
 *                   file on the local filesystem that needs to be uploaded.
 * @param info       A pointer to a platform_info structure that contains
 *                   platform-specific data that will be included in the
 *                   upload request.
 *
 * @return The function returns `WM_TRUE` if the upload is successful, and `WM_FALSE` if it fails.
 */
boolean_t upload_configuration(const char* server_url, const char* path, platform_info* info);

#endif