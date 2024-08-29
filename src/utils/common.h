#ifndef _UTILS_COMMON_H_
#define _UTILS_COMMON_H_

#include <stddef.h>
#include <stdint.h>
#include <unistd.h>
#include <time.h>

#define ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]))

#define STATIC_ASSERT _Static_assert

typedef enum {
    WM_FALSE = 0,
    WM_TRUE  = 1,
} boolean_t;

#endif
