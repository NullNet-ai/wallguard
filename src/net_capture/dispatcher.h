#ifndef _DISPATCHER_H_
#define _DISPATCHER_H_

#include <utils/dbuffer.h>
#include <net_capture/transmitter.h>
#include <pthread.h>
#include <sys/time.h>

/**
 * @brief Structure representing a dispatcher for buffering and transmitting data.
 */
struct dispatcher;

typedef struct dispatcher dispatcher_t;

/**
 * @brief Initializes a dispatcher instance.
 *
 * @param public_key The public key used to initialize the transmitter.
 * 
 * @return A pointer to the newly created dispatcher instance, or `NULL` on failure.
 */
dispatcher_t* dispatcher_initialize(const char* public_key);

/**
 * @brief Finalizes and cleans up a dispatcher instance.
 *
 * @param instance A pointer to the dispatcher instance to finalize.
 */
void dispatcher_finalize(dispatcher_t* instance);

/**
 * @brief Writes data to the dispatcher for buffering and transmission.
 *
 * @param instance A pointer to the dispatcher instance.
 * @param device   The name of the network device where the data was captured.
 * @param time     Time when the packed was captured.
 * @param data     A pointer to the data to write.
 * @param len      The length of the data to write.
 */
void dispatcher_write(dispatcher_t* instance, const char* device, struct timeval* time, const void* data, size_t len);

#endif