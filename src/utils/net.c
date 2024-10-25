#include "net.h"

#include <net/if.h>
#include <string.h>

boolean_t is_interface_valid(const char* ifname) {
    if (!ifname) {
        return WM_FALSE;
    }

    struct if_nameindex* if_index = if_nameindex();

    if (if_index == NULL) {
        return WM_FALSE;
    }

    boolean_t found = WM_FALSE;

    for (struct if_nameindex* iface = if_index; iface->if_index != 0 || iface->if_name != NULL; ++iface) {
        if (strcmp(iface->if_name, ifname) == 0) {
            found = WM_TRUE;
            break;
        }
    }

    if_freenameindex(if_index);
    return found;
}
