#include <utils/str.h>

#include <string.h>

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