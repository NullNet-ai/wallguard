#ifndef _UTILS_COMMON_H_
#define _UTILS_COMMON_H_

#include <stddef.h>
#include <stdint.h>
#include <unistd.h>
#include <time.h>
#include <stdlib.h>

#define ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]))

#define STATIC_ASSERT _Static_assert

typedef enum {
    WM_FALSE = 0,
    WM_TRUE  = 1,
} boolean_t;

/**
 * @brief Allocates memory of the given size and checks if the allocation was successful.
 * 
 * @param size The size of memory to allocate, in bytes.
 * @return A pointer to the allocated memory. The program terminates if allocation fails.
 */
void* __wallmon_malloc(size_t size);

/**
 * @brief Allocates memory for an array and initializes the memory to zero.
 * 
 * @param nmem The number of elements to allocate.
 * @param size The size of each element in bytes.
 * @return A pointer to the allocated and zero-initialized memory. The program terminates if allocation fails.
 */
void* __wallmon_calloc(size_t nmem, size_t size);

/**
 * @brief Resizes an existing memory block and checks if the reallocation was successful.
 * 
 * @param ptr The pointer to the memory block to resize. If NULL, realloc behaves like malloc.
 * @param size The new size for the memory block, in bytes.
 * @return A pointer to the reallocated memory. The program terminates if reallocation fails.
 */
void* __wallmon_realloc(void* ptr, size_t size);

#if 1
    #define W_MALLOC __wallmon_malloc
    #define W_CALLOC __wallmon_calloc
    #define W_REALLOC __wallmon_realloc
#else
    #define W_MALLOC malloc
    #define W_CALLOC calloc
    #define W_REALLOC realloc
#endif

#endif
