#ifndef PLATFORM_IDENT_H
#define PLATFORM_IDENT_H

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
    const char*   model;
    const char*   version;
    char          uuid[37];  // 36 chars for formatter UUID string + 1 for null-terminator
} platform_info;

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

#endif  // PLATFORM_IDENT_H
