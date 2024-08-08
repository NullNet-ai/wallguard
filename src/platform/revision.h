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

boolean_t obtain_revision(platform_type platform, revision* rev);

#endif
