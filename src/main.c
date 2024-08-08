#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "platform/revision.h"

int main(int argc, char** argv) {
    (void)argc;
    (void)argv;

    revision  r;
    boolean_t result = obtain_revision(PLATFORM_OPNSENSE, &r);
    if (result)
        printf("Rev time: %ld\nRev username: %s\n", r.time, r.username);
    else
        printf("Failed to obtain current revision\n");
    return EXIT_SUCCESS;
}