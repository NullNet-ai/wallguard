#ifndef PLATFORM_IDENT_H
#define PLATFORM_IDENT_H

/**
 * @brief Enumeration to represent different platform types.
 */
typedef enum
{
    PLATFORM_PFSENSE,
    PLATFORM_OPNSENSE,
    PLATFORM_UNDEFINED
} platform;

/**
 * @brief Identifies the current platform.
 *
 * @return platform Enum value indicating the platform type.
 */
platform ident();

/**
 * @brief Returns the name of the given platform.
 *
 * @param platform The platform enum value.
 * @return const char* A string representing the platform name.
 */
const char *platform_name(platform platform);

#endif // PLATFORM_IDENT_H