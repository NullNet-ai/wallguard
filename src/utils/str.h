#ifndef UTILS_STR_H
#define UTILS_STR_H

#include <stddef.h>

/**
 * @brief Generate a random string.
 *
 * @param buf A pointer to the buffer where the random string will be stored. The buffer must be large enough to hold
 * the string and the null terminator.
 * @param len The length of the random string to generate.
 */
void generate_random_string(char *buf, size_t len);

#endif
