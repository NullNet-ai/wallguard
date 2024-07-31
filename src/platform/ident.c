#include <platform/ident.h>

platform ident()
{
    return PLATFORM_UNDEFINED;
}

const char *platform_name(platform platform)
{
    switch (platform)
    {
    case PLATFORM_PFSENSE:
        return "pfSense";
    case PLATFORM_OPNSENSE:
        return "OPNsense";
    default:
        return "Undefined";
    }
}