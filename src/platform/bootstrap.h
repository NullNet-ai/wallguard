#ifndef _PLATFORM_BOOTSTRAP_H_
#define _PLATFORM_BOOTSTRAP_H_

#include <utils/common.h>
#include <platform/ident.h>

/**
 * @brief Checks if the system is currently locked.
 *
 * @return `WM_TRUE` if the lock file exists, otherwise `WM_FALSE`.
 */
boolean_t system_locked(void);

/**
 * @brief Locks the system by creating a lock file and writing the UUID to it.
 *
 * @param info A pointer to the `platform_info`.
 * @return `WM_TRUE` if the lock file was successfully created and written to,
 *                     otherwise `WM_FALSE`.
 */
boolean_t lock_system(platform_info* info);

/**
 * @brief Validates the lock by comparing the UUID in the lock file with the platform's UUID.
 *
 * @param info A pointer to the `platform_info` structure.
 * @return `WM_TRUE` if the UUIDs match, otherwise `WM_FALSE`.
 */
boolean_t validate_lock(platform_info* info);

#endif
