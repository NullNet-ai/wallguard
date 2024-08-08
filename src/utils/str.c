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
