#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "network/file_transfer.h"

#define BUFFER_SIZE 8192

int main(int argc, char** argv) {
    (void)argc;
    (void)argv;

    boolean_t r = upload("192.168.2.19", 3000, "/upload", "/home/anton/WallMon/README.md", WM_FALSE);

    printf("File uploaded: %d\n", r);

    return EXIT_SUCCESS;
}
