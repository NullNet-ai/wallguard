#ifndef UTILS_URL_H
#define UTILS_URL_H

#include <utils/common.h>

/**
 * @brief Parses a URL string and extracts the hostname, path, port, and whether the connection uses TLS.
 *
 * @param url The input URL string to be parsed.
 * @param hostname Buffer to store the extracted hostname.
 * @param hostname_maxlen Maximum length of the hostname buffer, including the null terminator.
 * @param path Buffer to store the extracted path.
 * @param path_maxsize Maximum size of the path buffer, including the null terminator.
 * @param port Pointer to an integer where the port number will be stored. If `NULL`, the port is not returned.
 * @param tls Pointer to a `boolean_t` value where the TLS status will be stored. `WM_TRUE` for HTTPS, `WM_FALSE` for
 * HTTP. If `NULL`, the TLS status is not returned.
 *
 * @return `WM_TRUE` if the URL was successfully parsed, `WM_FALSE` otherwise.
 */
boolean_t parse_url(const char* url, char* hostname, size_t hostname_maxlen, char* path, size_t path_maxsize, int* port,
                    boolean_t* tls);

#endif
