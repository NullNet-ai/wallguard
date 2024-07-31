#include <platform/pfsense.h>
#include <utils/file_utils.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int has_platform_file_with_corresponding_content()
{
    const char *path = "/etc/platform";
    if (!file_exists(path))
    {
        return 0;
    }

    FILE *file = fopen(path, "r");
    if (!file)
    {
        return 0;
    }

    char buffer[8];
    memset(buffer, 0, sizeof(buffer));

    fscanf(file, "%7s", buffer);
    fclose(file);

    return strcmp(buffer, "pfSense") == 0;
}

static int has_config_file()
{
    const char *path = "/conf/config.xml";
    return file_exists(path);
}

int is_pfsense()
{
    const int c0 = has_platform_file_with_corresponding_content();
    const int c1 = has_config_file();

    return c0 && c1;
}