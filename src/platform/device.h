#ifndef _PLATFORM_DEVICE_H_
#define _PLATFORM_DEVICE_H_

#include <utils/common.h>

/**
 * @brief Retrieves the UUID of the device.
 *
 * @param uuid A pointer to a buffer where the UUID will be stored.
 * @param size The size of the buffer.
 * @return Returns `WM_TRUE` on success, `WM_FALSE` on failure.
 */
boolean_t device_uuid(char* uuid, size_t size);

#endif
