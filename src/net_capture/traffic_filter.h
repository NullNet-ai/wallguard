#ifndef _TRAFFIC_FILTER_H_
#define _TRAFFIC_FILTER_H_

#include <utils/common.h>
#include <pcap.h>

/**
 * @brief Formats a traffic filter string.
 *
 * @param code A pointer to the buffer where the filter string will be written.
 * @param len The size of the buffer, including space for the null terminator.
 *
 * @return `WM_TRUE` if the filter string is successfully formatted, `WM_FALSE` otherwise.
 */
boolean_t format_filter(char* code, size_t len);

/**
 * @brief Compiles and applies a BPF filter to a pcap handle.
 *
 * @param handle A pointer to the pcap handle where the filter will be applied.
 * @param filter The filter string to compile and apply.
 *
 * @return A pointer to the compiled BPF program, or NULL if an error occurred.
 */
struct bpf_program* build_filter(pcap_t* handle, const char* filter);

#endif
