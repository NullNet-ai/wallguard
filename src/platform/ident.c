#include <platform/ident.h>
#include <utils/file_utils.h>
#include <utils/common.h>

/**
 * @brief List of files and directories that are likely to be found on a pfSense system
 */
static const char *pfsense_files[] = {
    "/conf/config.xml", 
    "/usr/local/pfSense", 
    "/etc/platform", 
    "/etc/version", 
    "/usr/local/sbin/pfSsh.php",
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

platform ident() {
    if (files_exist(pfsense_files, ARRAY_SIZE(pfsense_files))) {
        return PLATFORM_PFSENSE;
    }

    if (files_exist(opnsense_files, ARRAY_SIZE(opnsense_files))) {
        return PLATFORM_OPNSENSE;
    }

    return PLATFORM_UNSUPPORTED;
}

const char *platform_name(platform platform) {
    switch (platform) {
        case PLATFORM_PFSENSE:
            return "pfSense";
        case PLATFORM_OPNSENSE:
            return "OPNsense";
        default:
            return "Unsupported";
    }
}
