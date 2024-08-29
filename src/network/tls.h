#ifndef _NETWORK_TLS_H_
#define _NETWORK_TLS_H_

#include <utils/common.h>

typedef struct tls_handle tls_handle;

/**
 * @brief Starts a TLS connection.
 *
 * @param handle A pointer to a `tls_handle` pointer that will be allocated and initialized.
 * @param hostname The hostname to connect to.
 * @param port The port number to connect to.
 * @return `WM_TRUE` on success, `WM_FALSE` on failure.
 */
boolean_t tls_start(tls_handle** handle, const char* hostname, int port);

/**
 * @brief Writes data to the TLS connection.
 *
 * @param handle The TLS handle.
 * @param data A pointer to the data to be sent.
 * @param len The length of the data to be sent.
 * @return The number of bytes written on success, `-1` on failure.
 */
ssize_t tls_write(tls_handle* handle, uint8_t* data, size_t len);

/**
 * @brief Reads data from the TLS connection.
 *
 * @param handle The TLS handle.
 * @param data A pointer to the buffer to store the read data.
 * @param len The length of the buffer.
 * @return The number of bytes read on success, `-1` on failure.
 */
ssize_t tls_read(tls_handle* handle, uint8_t* data, size_t len);

/**
 * @brief Ends the TLS connection.
 *
 * @param handle The TLS handle.
 * @return `WM_TRUE` on success, `WM_FALSE` on failure.
 */
boolean_t tls_end(tls_handle* handle);

#endif
