#ifndef NETWORK_FILE_TRANSFER_H
#define NETWORK_FILE_TRANSFER_H

#include <utils/common.h>

/**
 * @brief Loads a file from an HTTP server and saves it to a local file.
 *
 * @param hostname The hostname of the HTTP server.
 * @param port The port number of the HTTP server.
 * @param file_path The path of the file on the HTTP server.
 * @param local_file The path of the local file where the content will be saved.
 * @param tls If `WM_TRUE`, a TLS connection is used; otherwise, a TCP connection is used.
 * @return `WM_TRUE` on success, `WM_FALSE` on failure.
 */
boolean_t download(const char* hostname, int port, const char* file_path, const char* local_file, boolean_t tls);

/**
 * @brief Uploads a local file to an HTTP server.
 *
 * @param hostname The hostname of the HTTP server.
 * @param port The port number of the HTTP server.
 * @param path The server path.
 * @param local_file The path of the local file to be uploaded.
 * @param tls If `WM_TRUE`, a TLS connection is used; otherwise, a TCP connection is used.
 * @return `WM_TRUE` on success, `WM_FALSE` on failure.
 */
boolean_t upload(const char* hostname, int port, const char* path, const char* local_file, boolean_t tls);

#endif
