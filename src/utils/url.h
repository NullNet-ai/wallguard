#ifndef UTILS_URL_H
#define UTILS_URL_H

#include <utils/common.h>

/**
 * @brief Parses a URL string to extract the hostname, port, and whether the connection is secure (TLS).
 *
 * Supports only HTTP and HTTPS protocols.
 *
 * @param url The URL string to parse.
 * @param hostname A buffer to store the extracted hostname. This buffer should be large enough to hold the hostname.
 * @param port A pointer to an integer to store the extracted port number. Defaults to `80` for HTTP and `443` for HTTPS
 * if not specified.
 * @param tls A pointer to a `boolean_t` to indicate if the URL uses HTTPS (`WM_TRUE`) or HTTP (`WM_FALSE`).
 * @return Returns `WM_TRUE` if the URL was successfully parsed, `WM_FALSE` if the URL format was invalid.
 */
boolean_t parse_url(const char* url, char* hostname, char* path, int* port, boolean_t* tls);

#endif
