#include <utils/str.h>

#include <string.h>
#include <stdlib.h>
#include <time.h>

static const char charset[] = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

void generate_random_string(char* buf, size_t len) {
    if (!buf) {
        return;
    }

    srand(time(NULL));

    for (size_t i = 0; i < len - 1; i++) {
        buf[i] = charset[rand() % (sizeof(charset) - 1)];
    }

    buf[len - 1] = '\0';
}

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
