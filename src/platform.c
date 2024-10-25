#include <utils/file_utils.h>

#include "platform.h"
#include "config.h"

#include <stdlib.h>
#include <string.h>

platform_type get_platform_type() {
    const char *platform = cfg_get_platform();

    if (strcmp(platform, "pfSense") == 0) {
        return PLATFORM_PFSENSE;
    }

    if (strcmp(platform, "OPNsense") == 0) {
        return PLATFORM_OPNSENSE;
    }

    return PLATFORM_UNSUPPORTED;
}

static const char *get_platform_version(platform_type platform) {
    switch (platform) {
        case PLATFORM_OPNSENSE: {
            char *str = (char *)read_file_content("/usr/local/opnsense/version/base");

            // I forgot what this piece is for
            if (str != NULL) {
                size_t len = strcspn(str, "\n");
                size_t tln = strlen(str);

                if (len < tln) {
                    str[len] = '\0';
                }
            }

            return str;
        }
        case PLATFORM_PFSENSE: {
            char *str = (char *)read_file_content("/etc/version");

            if (str != NULL) {
                size_t len = strcspn(str, "\n");
                size_t tln = strlen(str);

                if (len < tln) {
                    str[len] = '\0';
                }
            }

            return str;
        }
        default:
            return NULL;
    }
}

platform_info *get_platform_info() {
    platform_type type = get_platform_type();
    if (type == PLATFORM_UNSUPPORTED) {
        return NULL;
    }

    const char *version = get_platform_version(type);
    if (!version) {
        return NULL;
    }

    platform_info *info = malloc(sizeof(platform_info));
    if (!info) {
        free((void *)version);
        return NULL;
    }

    info->type    = type;
    info->version = version;

    info->model = cfg_get_platform();
    info->uuid  = cfg_get_system_uuid();

    return info;
}

void release_platform_info(platform_info *info) {
    if (info == NULL) {
        return;
    }

    free((void *)info->version);
    free((void *)info);
}
