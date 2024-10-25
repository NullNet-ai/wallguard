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

#endif
