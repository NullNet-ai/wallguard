#include <utils/common.h>
#include <utils/file_utils.h>

#include "platform.h"
#include "config.h"

#include <stdlib.h>
#include <string.h>
#include <dirent.h>

// @TODO: The implmentation might differ per platform
static boolean_t is_system_dirty() {
    DIR *directory = opendir("/var/run/");
    if (!directory) {
        return WM_FALSE;
    }

    int8_t retval = WM_FALSE;

    struct dirent *info;
    while ((info = readdir(directory)) != NULL) {
        const char *ext = extension(info->d_name);
        if (ext && strcmp(ext, "dirty") == 0) {
            retval = WM_TRUE;
            break;
        }
    }

    closedir(directory);
    return retval;
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

platform_info *get_platform_info() {
    platform_type type = get_platform_type();
    if (type == PLATFORM_UNSUPPORTED) {
        return NULL;
    }

    const char *version = get_platform_version(type);
    if (!version) {
        return NULL;
    }

    platform_info *info = W_MALLOC(sizeof(platform_info));

    info->type    = type;
    info->version = version;

    info->dirty = is_system_dirty();

    info->model         = cfg_get_platform();
    info->uuid          = cfg_get_system_uuid();
    info->instance_name = cfg_get_instance_name();

    return info;
}

void release_platform_info(platform_info *info) {
    if (info == NULL) {
        return;
    }

    free((void *)info->version);
    free((void *)info);

    info->model         = NULL;
    info->uuid          = NULL;
    info->instance_name = NULL;
}

void update_platform_info(platform_info *info) {
    info->dirty = is_system_dirty();
    // Update version ?
}
