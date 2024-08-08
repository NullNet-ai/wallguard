#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "platform/revision.h"

int main(int argc, char** argv) {
    (void)argc;
    (void)argv;

    revision r;
    obtain_revision(PLATFORM_OPNSENSE, &r);

    printf("Rev time: %ld\nRev username: %s\n", r.time, r.username);

    return EXIT_SUCCESS;
}