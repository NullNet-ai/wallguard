#include <utils/str.h>

#include <string.h>
#include <stdlib.h>

boolean_t string_to_integer(const char* str, long* number, int base) {
    char* endptr = NULL;
    *number      = strtol(str, &endptr, base);

    if (endptr == str) {
        return WM_FALSE;
    }

    if (*endptr != 0) {
        return WM_FALSE;
    }

    return WM_TRUE;
}
