#ifndef _SNIFFER_H_
#define _SNIFFER_H

#include <utils/common.h>

typedef struct sniffer sniffer_t;

/**
 * @brief Initializes the sniffer module.
 *
 * @param public_key Public key of the monitoring server.
 *
 * @return A pointer to the initialized `sniffer_t` structure, or `NULL` if initialization fails.
 */
sniffer_t* sniffer_initialize(const char* public_key);

/**
 * @brief Starts the main loop for the sniffer.
 *
 * @param sniffer A pointer to the `sniffer_t` structure initialized by `sniffer_initialize()`.
 */
void sniffer_mainloop(sniffer_t* sniffer);

/**
 * @brief Finalizes the sniffer module.
 *
 * @param sniffer A pointer to the `sniffer_t` structure initialized by `sniffer_initialize()`.
 */
void sniffer_finalize(sniffer_t* sniffer);

#endif
