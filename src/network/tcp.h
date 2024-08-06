#ifndef NETWORK_TCP_H
#define NETWORK_TCP_H

#include <utils/common.h>

typedef struct tcp_handle tcp_handle;

/**
 * @brief Starts a TCP connection.
 *
 * @param handle A pointer to a `tcp_handle` pointer that will be allocated and initialized.
 * @param hostname The hostname to connect to.
 * @param port The port number to connect to.
 * @return `WM_TRUE` on success, `WM_FALSE` on failure.
 */
boolean_t tcp_start(tcp_handle** handle, const char* hostname, int port);

/**
 * @brief Writes data to the TCP connection.
 *
 * @param handle The TCP handle.
 * @param data A pointer to the data to be sent.
 * @param len The length of the data to be sent.
 * @return The number of bytes written on success, `-1` on failure.
 */
ssize_t tcp_write(tcp_handle* handle, uint8_t* data, size_t len);

/**
 * @brief Reads data from the TCP connection.
 *
 * @param handle The TCP handle.
 * @param data A pointer to the buffer to store the read data.
 * @param len The length of the buffer.
 * @return The number of bytes read on success, `-1` on failure.
 */
ssize_t tcp_read(tcp_handle* handle, uint8_t* data, size_t len);

/**
 * @brief Ends the TCP connection.
 *
 * @param handle The TCP handle.
 * @return `WM_TRUE` on success, `WM_FALSE` on failure.
 */
boolean_t tcp_end(tcp_handle* handle);

#endif
