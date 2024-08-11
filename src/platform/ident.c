#include <platform/ident.h>
#include <platform/device.h>

#include <utils/file_utils.h>
#include <utils/common.h>

#include <stdlib.h>

/**
 * @brief List of files and directories that are likely to be found on a pfSense system
 */
static const char *pfsense_files[] = {
    "/conf/config.xml", "/usr/local/pfSense", "/etc/platform", "/etc/version", "/usr/local/sbin/pfSsh.php",
};

/**
 * @brief List of files and directories that are likely to be found on a OPNsense system
 */
static const char *opnsense_files[] = {
    "/conf/config.xml",
    "/usr/local/opnsense",
    "/usr/local/opnsense/version",
    "/usr/local/sbin/pluginctl",
};

static platform_type get_platform_type() {
    if (files_exist(pfsense_files, ARRAY_SIZE(pfsense_files))) {
        return PLATFORM_PFSENSE;
    }

    if (files_exist(opnsense_files, ARRAY_SIZE(opnsense_files))) {
        return PLATFORM_OPNSENSE;
    }

    return PLATFORM_UNSUPPORTED;
}

static const char *get_platform_version(platform_type platform) {
    switch (platform) {
        case PLATFORM_OPNSENSE:
            return NULL;

        case PLATFORM_PFSENSE:
            return (const char *)read_file_content("/etc/version");

        default:
            return NULL;
    }
}

static const char *get_platform_model(platform_type platform) {
    switch (platform) {
        case PLATFORM_PFSENSE:
            return "pfSense";
        case PLATFORM_OPNSENSE:
            return "OPNsense";
        default:
            return NULL;
    }
}

platform_info *get_platform_info() {
    platform_type type = get_platform_type();
    if (type == PLATFORM_UNSUPPORTED) {
        return NULL;
    }

    const char *model = get_platform_model(type);
    if (!model) {
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
    info->model   = model;
    info->version = version;

    if (!device_uuid(info->uuid, sizeof(info->uuid))) {
        free((void *)version);
        free((void *)info);
        return WM_FALSE;
    }

    return info;
}

void release_platform_info(platform_info *info) {
    if (info == NULL) {
        return;
    }

    free((void *)info->version);
    free((void *)info);
}
