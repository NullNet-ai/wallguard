#ifndef _NET_REQUEST_H_
#define _NET_REQUEST_H_

#include <utils/common.h>

typedef struct request_handle request_handle;

/**
 * @brief Starts a network request connection (TCP or TLS).
 *
 * @param handle A pointer to a request_handle pointer that will be allocated and initialized.
 * @param hostname The hostname to connect to.
 * @param port The port number to connect to.
 * @param tls If WM_TRUE, a TLS connection is established; otherwise, a TCP connection is established.
 * @return `WM_TRUE` on success, `WM_FALSE` on failure.
 */
boolean_t request_start(request_handle** handle, const char* hostname, int port, boolean_t tls);

/**
 * @brief Writes data to the network connection.
 *
 * @param handle The request handle.
 * @param data A pointer to the data to be sent.
 * @param len The length of the data to be sent.
 * @return The number of bytes written on success, `-1` on failure.
 */
ssize_t request_write(request_handle* handle, uint8_t* data, size_t len);

/**
 * @brief Reads data from the network connection.
 *
 * @param handle The request handle.
 * @param data A pointer to the buffer to store the read data.
 * @param len The length of the buffer.
 * @return The number of bytes read on success, `-1` on failure.
 */
ssize_t request_read(request_handle* handle, uint8_t* data, size_t len);

/**
 * @brief Ends the network connection.
 *
 * @param handle The request handle.
 */
void request_end(request_handle* handle);

#endif
