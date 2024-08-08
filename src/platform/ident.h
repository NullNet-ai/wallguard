#ifndef PLATFORM_IDENT_H
#define PLATFORM_IDENT_H

/**
 * @brief Enumeration to represent different platform types.
 */
typedef enum {
    PLATFORM_PFSENSE,
    PLATFORM_OPNSENSE,
    PLATFORM_UNSUPPORTED,
} platform_type;

/**
 * @brief Identifies the current platform.
 *
 * @return `platform_type` enum value indicating the platform type.
 */
platform_type ident();

/**
 * @brief Returns the name of the given platform.
 *
 * @param platform The `platform_type` enum value.
 * @return const char* A string representing the platform name.
 */
const char *platform_name(platform_type platform);

#endif  // PLATFORM_IDENT_H
