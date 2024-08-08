#ifndef PLATFORM_REVISION_H
#define PLATFORM_REVISION_H

#include <platform/ident.h>
#include <utils/common.h>

// @TODO: Is 256 enough?
#define USERNAME_LENGTH 256

typedef struct {
    char   username[USERNAME_LENGTH];
    time_t time;
} revision;

/**
 * @brief Obtains the revision information based on the platform type.
 *
 * @param platform The type of platform (PF_SENSE or OPN_SENSE).
 * @param rev Pointer to the revision structure to store the parsed data.
 * @return `WM_TRUE` if the revision information was successfully obtained, `WM_FALSE` otherwise.
 */
boolean_t obtain_revision(platform_type platform, revision* rev);

#endif
