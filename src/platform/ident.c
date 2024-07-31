#include <platform/ident.h>
#include <platform/pfsense.c>

platform ident()
{
    if (is_pfsense())
    {
        return PLATFORM_PFSENSE;
    }

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