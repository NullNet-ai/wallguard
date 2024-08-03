#ifndef UTILS_COMMON_H
#define UTILS_COMMON_H

#include <stddef.h>

#define ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]))

typedef enum {
    WM_FALSE = 0,
    WM_TRUE  = 1,
} boolean_t;

#endif
