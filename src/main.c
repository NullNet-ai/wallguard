#include <stdlib.h>
#include <stdio.h>

#include "platform/ident.h"

int main(int argc, char **argv)
{
    (void)argc;
    (void)argv;

    const platform pl = ident();
    const char *pl_name = platform_name(pl);

    printf("Identified platform:\t%s\n", pl_name);
    return EXIT_SUCCESS;
}