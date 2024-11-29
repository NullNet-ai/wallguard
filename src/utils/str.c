#include <utils/str.h>

#include <string.h>
#include <stdlib.h>
#include <ctype.h>

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

boolean_t is_valid_uuid(const char* value) {
    if (!value) {
        return WM_FALSE;
    }

    if (strlen(value) != 36) {
        return WM_FALSE;
    }

    for (size_t i = 0; i < 36; ++i) {
        if (i == 8 || i == 13 || i == 18 || i == 23) {
            if (value[i] != '-') {
                return WM_FALSE;
            }
        } else {
            if (!isxdigit(value[i])) {
                return WM_FALSE;
            }
        }
    }

    return WM_TRUE;
}

char* string_copy(const char* str) {
    size_t len  = strlen(str) + 1;
    char*  copy = W_MALLOC(len);
    memcpy(copy, str, len);
    return copy;
}
