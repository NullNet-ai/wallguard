#include "cli_args.h"

#include <getopt.h>
#include <stdio.h>
#include <stdlib.h>
#include <utils/common.h>

#ifndef VERSION
#define VERSION "unknown"
#endif

struct cli_args cli_args;

static void print_usage(const char* exec_name, boolean_t print_options) {
    printf("Usage: %s -c <config filename> [-hv]\n", exec_name);

    if (!print_options) {
        return;
    }

    printf("\nOptions:\n");
    printf("  -c <config filename>     Path to the configuration file\n");
    printf("  -v                       Display the version\n");
    printf("  -h                       Display this help message\n");
}

void parse_cli_arguments(int argc, char** argv) {
    cli_args.config_filename = NULL;

    int32_t opt;
    while ((opt = getopt(argc, argv, "hvc:")) != -1) {
        switch (opt) {
            case 'c':
                cli_args.config_filename = optarg;
                break;
            case 'h':
                print_usage(argv[0], WM_TRUE);
                exit(EXIT_SUCCESS);
            case 'v':
                printf("%s\n", VERSION);
                exit(EXIT_SUCCESS);
            default:
                print_usage(argv[0], WM_TRUE);
                exit(EXIT_FAILURE);
        }
    }

    if (!cli_args.config_filename) {
        print_usage(argv[0], WM_FALSE);
        exit(EXIT_FAILURE);
    }
}