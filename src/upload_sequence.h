#ifndef _UPLOAD_SEQUENCE_H_
#define _UPLOAD_SEQUENCE_H_

#include <utils/common.h>
#include "platform.h"

/**
 * Encrypts and uploads a configuration file securely.
 *
 * This function performs the following steps:
 *   1. Generates a temporary filename.
 *   2. Fetches a public key using a session token.
 *   3. Encrypts the configuration file, using AES encryption with a randomly generated key and IV.
 *   4. Encrypts the AES key and IV with the public key.
 *   5. Base64 encodes the encrypted AES key and IV.
 *   6. Uploads the encrypted file and encoded key/IV to the server.
 *
 * @param session_token Pointer to a string containing the session token for authentication.
 * @param path          Pointer to a string with the file path of the configuration file to be uploaded.
 * @param info          Pointer to a `platform_info` struct.
 *
 * @return `WM_TRUE` on success, or `WM_FALSE` on failure.
 */
boolean_t upload_sequence(const char* session_token, const char* path, const platform_info* info);

#endif