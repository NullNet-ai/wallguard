#ifndef _TRANSMITTER_H_
#define _TRANSMITTER_H_

#include <utils/common.h>

typedef struct transmitter transmitter_t;

/**
 * @brief Initializes a transmitter for sending data to a specified address.
 *
 * @param server_key Server's public key
 * 
 * @return A pointer to the initialized transmitter instance, or `NULL` if initialization fails.
 */
transmitter_t* transmitter_initialize(const char* server_key);

/**
 * @brief Sends data through the transmitter.
 *
 * @param handle A pointer to the initialized transmitter instance.
 * @param data A pointer to the data buffer to be sent.
 * @param len The length of the data buffer, in bytes.
 * @return `WM_TRUE` if the data is sent successfully, `WM_FALSE` otherwise.
 */
boolean_t transmitter_send(transmitter_t* handle, void* data, size_t len);

/**
 * @brief Finalizes the transmitter and cleans up allocated resources.
 *
 * @param handle A pointer to the transmitter instance to be finalized.
 */
void transmitter_finalize(transmitter_t* handle);

#endif
