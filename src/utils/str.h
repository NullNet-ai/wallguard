#ifndef _UTILS_STR_H_
#define _UTILS_STR_H_

#include <stddef.h>
#include <utils/common.h>

/**
 * @brief Generate a random string.
 *
 * @param buf A pointer to the buffer where the random string will be stored. The buffer must be large enough to hold
 * the string and the null terminator.
 * @param len The length of the random string to generate.
 */
void generate_random_string(char *buf, size_t len);

boolean_t string_to_integer(const char* str, long* number, int base);

#endif
