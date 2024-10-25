#ifndef _UTILS_NET_H_
#define _UTILS_NET_H_

#include <utils/common.h>

/**
 * @brief Checks if a given network interface name is valid on the system.
 *
 * @param ifname  The name of the network interface to be validated (e.g., "eth0", "wlan0").
 *
 * @return        Returns `WM_TRUE` if the network interface exists on the system, otherwise returns `WM_FALSE`.
 */
boolean_t is_interface_valid(const char* ifname);

#endif
