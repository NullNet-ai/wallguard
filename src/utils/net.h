#ifndef _UTILS_NET_H_
#define _UTILS_NET_H_

#include <utils/common.h>
#include <arpa/inet.h>

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

/**
 * @brief Checks if a given network interface name is valid on the system.
 *
 * @param ifname  The name of the network interface to be validated (e.g., "eth0", "wlan0").
 *
 * @return        Returns `WM_TRUE` if the network interface exists on the system, otherwise returns `WM_FALSE`.
 */
boolean_t is_interface_valid(const char* ifname);

/**
 * @brief Resolves a given hostname to an IPv4 address.
 *
 * @param hostname The hostname to resolve (e.g., "example.com").
 * @param address A buffer to store the resolved IPv4 address. Must be at least `INET_ADDRSTRLEN` bytes long.
 * @return Returns `WM_TRUE` if the resolution succeeds,`WM_FALSE` otherwise.
 */
boolean_t resolve_hostname_v4(const char* hostname, char address[INET_ADDRSTRLEN]);

/**
 * @brief Resolves a given hostname to an IPv6 address.
 *
 * @param hostname The hostname to resolve (e.g., "example.com").
 * @param address A buffer to store the resolved IPv6 address. Must be at least `INET6_ADDRSTRLEN` bytes long.
 * @return Returns `WM_TRUE` if the resolution succeeds,`WM_FALSE` otherwise.
 */
boolean_t resolve_hostname_v6(const char* hostname, char address[INET6_ADDRSTRLEN]);

#endif
