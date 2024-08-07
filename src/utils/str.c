#include <utils/str.h>

#include <string.h>
#include <stdlib.h>
#include <time.h>

static const char charset[] = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

void generate_random_string(char *buf, size_t len) {
    if (!buf) {
        return;
    }

    srand(time(NULL));

    for (size_t i = 0; i < len - 1; i++) {
        buf[i] = charset[rand() % (sizeof(charset) - 1)];
    }

    buf[len - 1] = '\0';
}

size_t count_substring_occurrences(const char *str, const char *sub) {
    if (!str || !sub) {
        return 0;
    }

    size_t      retval  = 0;
    const char *tmp     = str;
    int         sub_len = strlen(sub);

    while ((tmp = strstr(tmp, sub)) != NULL) {
        retval++;
        tmp += sub_len;
    }

    return retval;
}
