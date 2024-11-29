#ifndef _DBUFFER_H_
#define _DBUFFER_H_

#include <utils/common.h>

/**
 * @brief Structure representing a buffer with a specified capacity.
 */
typedef struct {
    size_t capacity;
    size_t offset;
    void*  memory;
} buffer_t;

/**
 * @brief Initializes a buffer with a given capacity.
 *
 * @param buff Pointer to the buffer structure to initialize.
 * @param capacity Size of the memory to allocate for the buffer.
 */
void buffer_init(buffer_t* buff, size_t capacity);

/**
 * @brief Frees the memory associated with a buffer and resets its properties.
 *
 * @param buff Pointer to the buffer structure to free.
 */
void buffer_free(buffer_t* buff);

/**
 * @brief Writes data into the buffer if there is enough capacity.
 *
 * @param buff Pointer to the buffer structure.
 * @param data Pointer to the data to be written.
 * @param len Size of the data to write.
 * @return `WM_TRUE` if the write is successful, `WM_FALSE` if there is not enough space.
 */
boolean_t buffer_write(buffer_t* buff, const void* data, size_t len);

/**
 * @brief Checks if the buffer can accommodate the specified length of data.
 *
 * @param buffer Pointer to the buffer structure.
 * @param len Size of the data to check.
 * @return `WM_TRUE` if the data can fit, `WM_FALSE` otherwise.
 */
boolean_t buffer_can_write(buffer_t* buff, size_t len);

/**
 * @brief Checks if the buffer is empty.
 *
 * @param buffer Pointer to the buffer structure.
 * @return `WM_TRUE` if the buffer is empty, `WM_FALSE` otherwise.
 */
boolean_t buffer_is_empty(buffer_t* buff);

/**
 * @brief Resets the current buffer's offset to 0.
 *
 * @param buffer Pointer to the buffer structure.
 */
void buffer_clear(buffer_t* buff);

#endif
