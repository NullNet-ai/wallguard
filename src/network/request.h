#ifndef NET_REQUEST_H
#define NET_REQUEST_H

#include <utils/common.h>

/**
 * @brief Establishes a TLS connection to the specified hostname and port, sends data, and returns the response.
 *
 * This function initializes the TLS library, creates a TLS context, and establishes a secure connection
 * to the specified hostname and port. It then sends the provided data and reads the server's response into
 * a dynamically allocated buffer, which is returned to the caller. The buffer is null-terminated.
 *
 * @param hostname The hostname to connect to.
 * @param port The port number to connect to.
 * @param data The data to send.
 * @param len The length of the data to send.
 * @return A pointer to the buffer containing the server's response. The caller is responsible for freeing this buffer.
 *         Returns `NULL` on error.
 */
uint8_t* request_tls(const char* hostname, int port, uint8_t* data, size_t len);

/**
 * @brief Establishes a TCP connection to the specified hostname and port, sends data, and returns the response.
 *
 * This function establishes a TCP connection to the specified hostname and port. It then sends the provided
 * data and reads the server's response into a dynamically allocated buffer, which is returned to the caller.
 * The buffer is null-terminated.
 *
 * @param hostname The hostname to connect to.
 * @param port The port number to connect to.
 * @param data The data to send.
 * @param len The length of the data to send.
 * @return A pointer to the buffer containing the server's response. The caller is responsible for freeing this buffer.
 *         Returns `NULL` on error.
 */
uint8_t* request_tcp(const char* hostname, int port, uint8_t* data, size_t len);

#endif
