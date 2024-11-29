#ifndef _UTILS_STR_H_
#define _UTILS_STR_H_

#include <stddef.h>
#include <utils/common.h>

/**
 * @brief Converts a string to a long integer.
 *
 * @param  str    The input string that represents a number.
 * @param  number A pointer to a long where the converted value will be stored.
 * @param  base   The base to use for the conversion.
 *
 * @return        Returns `WM_TRUE` if the conversion succeeds, otherwise returns `WM_FALSE`.
 */
boolean_t string_to_integer(const char* str, long* number, int base);

/**
 * @brief Checks if the given string is a valid UUID.
 *
 * @param  value  The string to be validated as a UUID.
 *
 * @return        Returns `WM_TRUE` if the string is a valid UUID, otherwise returns `WM_FALSE`.
 */
boolean_t is_valid_uuid(const char* value);

/**
 * @brief Checks if the given string is a valid UUID.
 *
 * @param value A valid UUID string.
 * @param bytes Output array where bytes will be stored.
 */
void uuid_to_bytes(const char uuid[36], uint8_t bytes[16]);

/**
 * @brief Creates a copy of the given null-terminated string.
 *
 * @param str A pointer to the null-terminated string to copy.
 * 
 * @return A pointer to the newly allocated string containing a copy of the input string.
 */
char* string_copy(const char* str);

#endif
