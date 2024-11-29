#ifndef _IFACE_LISTENER_H_
#define _IFACE_LISTENER_H_

#include <pcap/pcap.h>
#include <sys/time.h>
#include <utils/linked_list.h>

/**
 * @brief Callback function type for processing captured packet header data.
 *
 * @param device The name of the network interface where the packet was captured.
 * @param data   Pointer to the captured packet header data.
 * @param len    Length of the captured packet header data.
 */
typedef void (*write_callback_t)(const char* device, struct timeval* time, const void* data, size_t len);

/**
 * @brief Structure representing arguments for an interface listener routine.
 */
typedef struct {
    pcap_t*             handle;
    const char*         device;
    struct bpf_program* filter;
    write_callback_t    callback;
} iface_listener_info_t;

/**
 * @brief Thread routine function for listening to packets on a specific interface.
 *
 * @param arg A pointer to an `iface_listener_info_t` structure containing the
 *            necessary resources and parameters for the listener.
 * @return A pointer to the result of the routine execution.
 */
void* iface_listener_routine(void* arg);

/**
 * @brief Builds a list of interface listeners for packet capture.
 *
 * @param buffer_size The size of the pcap internal capture buffer.
 * @param callback    Callback function to handle captured packets.
 *
 * @return A linked list of initialized interface listeners, or NULL if an error occurs.
 */
llist_t* build_listeners_list(size_t buffer_size, write_callback_t callback);

/**
 * @brief Frees all resources associated with a list of interface listeners.
 *
 * @param list Pointer to the linked list of listeners.
 */
void free_listeners_list(llist_t* list);
#endif
