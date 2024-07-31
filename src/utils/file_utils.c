#include <utils/file_utils.h>
#include <unistd.h>

int file_exists(const char *path)
{
    return access(path, F_OK) == 0;
}