#ifndef PLATFORM_DEVICE_H
#define PLATFORM_DEVICE_H

#include <stddef.h>

/**
 * @brief Retrieves the UUID of the device.
 *
 * @param uuid A pointer to a buffer where the UUID will be stored.
 * @param size The size of the buffer.
 * @return int Returns 1 on success, 0 on failure.
 */
int device_uuid(char* uuid, size_t size);

#endif
