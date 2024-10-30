#ifndef _PLATFORM_H_
#define _PLATFORM_H_

/**
 * @brief Enum representing different supported platforms.
 */
typedef enum {
    PLATFORM_PFSENSE,
    PLATFORM_OPNSENSE,
    PLATFORM_UNSUPPORTED,
} platform_type;

/**
 * @brief Structure holding information about the platform.
 */
typedef struct {
    platform_type type;
    boolean_t     dirty;
    const char*   version;
    const char*   model;
    const char*   uuid;
    const char*   instance_name;
} platform_info;

/**
 * @brief Determines the platform type based on the configuration value.
 *
 * @return `platform_type` enum value
 */
platform_type get_platform_type();

/**
 * Identifies the current platform by examining specific system characteristics.
 *
 * @return A pointer to a `platform_info` structure containing information
 *         about the platform. The caller is responsible for freeing this
 *         structure using the `release_platform_info` function.
 */
platform_info* get_platform_info();

/**
 * Frees the memory allocated for the `platform_info` structure.
 *
 * @param info A pointer to the `platform_info` structure to be freed.
 */
void release_platform_info(platform_info* info);

/**
 * @brief Updates the platform information structure with the current system state.
 *
 * @param info Pointer to a `platform_info` structure.
 */
void update_platform_info(platform_info* info);

#endif
