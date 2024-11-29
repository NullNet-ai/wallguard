#ifndef _SERVER_REQUESTS_H_
#define _SERVER_REQUESTS_H_

#include <utils/common.h>
#include "platform.h"

/**
 * @brief Authenticates the client and allocates memory for the session token.
 *
 * @param session_token A pointer to a char pointer (i.e., `char**`), where the
 *                      function will assign the dynamically allocated session token.
 *                      It is the caller's responsibility to free this memory
 *                      after use.
 *
 * @return Returns `WM_TRUE` if authentication is successful, and `WM_FALSE`otherwise.
 *         In case of failure, no memory will be allocated.
 */
boolean_t wallmon_authenticate(char** session_token);

/**
 * @brief Sends a request to the central server with the current platform info.
 *
 * @param session_token A pointer to a `session_token`.
 * @param info          A pointer to a `platform_info` structure containing details about the current platform.
 * 
 * @return `WM_TRUE` if the request is successful, `WM_FALSE` if the request fails.
 */
boolean_t wallmon_setup(const char* session_token, const platform_info* info);

/**
 * @brief Requests the public key from the server.
 *
 * @param session_token A pointer to a `session_token`.
 * @param public_key    A pointer to a char pointer (i.e., `char**`), where the
 *                      function will assign the dynamically allocated public key.
 *                      It is the caller's responsibility to free this memory
 *                      after use.
 * @return `WM_TRUE` if the registration is successful, `WM_FALSE` if the registration fails.
 */
boolean_t wallmon_fetch_key(const char* session_token, char** public_key);

/**
 * @brief Sends a heartbeat signal to the central server to indicate that the system is still active.
 *
 * @param session_token A pointer to a `session_token`.
 * @return `WM_TRUE` if the request is successful, `WM_FALSE` otherwise.
 */
boolean_t wallmon_heartbeat(const char* session_token);

/**
 * @brief Uploads a configuration file to the server.
 *
 * @param session_token The authentication token for authorizing the upload request.
 * @param path          The file path to the configuration file that will be uploaded.
 * @param key           The encryption key required to decrypt the configuration file.
 * @param iv            The initialization vector (IV) needed along with the key for decryption.
 * @param info          Pointer to `platform_info` structure, containing platform state.
 * 
 * @return boolean_t    Returns `WM_TRUE` if the upload succeeded (HTTP status indicates success),
 *                      or `WM_FALSE` if the upload failed.
 */
boolean_t wallmon_upload_configuration(const char*          session_token,  //
                                       const char*          path,           //
                                       const char*          key,            //
                                       const char*          iv,             //
                                       const platform_info* info);


/**
 * @brief Requests the public key of the monitor server.
 * 
 * @param session_token A pointer to a `session_token`.
 * @param public_key    A pointer to a char pointer (i.e., `char**`), where the
 *                      function will assign the dynamically allocated public key.
 *                      It is the caller's responsibility to free this memory
 *                      after use.
 * 
 * @return `WM_TRUE` if the request is successful, `WM_FALSE` if the request fails.
 */
boolean_t wallmon_fetch_monitor_key(const char* session_token, char **public_key);

#endif
