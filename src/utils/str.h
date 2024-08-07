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

/**
 * @brief Counts the number of occurrences of a substring within a given string.
 *
 * @param str The input string in which to search for the substring.
 *            This string must be null-terminated.
 * @param sub The substring to search for within the input string.
 *            This string must be null-terminated.
 * @return The number of times the substring occurs in the input string.
 *         If either `str` or `sub` is `NULL`, the function returns `0`.
 */
size_t count_substring_occurrences(const char *str, const char *sub);

#endif
