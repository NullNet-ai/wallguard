#include "cli_args.h"

#include <getopt.h>
#include <stdio.h>
#include <stdlib.h>
#include <utils/common.h>

struct cli_args cli_args;

static void print_usage(const char* exec_name, boolean_t print_options) {
    printf("Usage: %s -u <server URL> [-i <network interface>] [-t <update interval>] [-s <system uuid>] [-h]\n",
           exec_name);

    if (!print_options) {
        return;
    }

    printf("\nOptions:\n");
    printf("  -u <server URL>          URL of the server to connect to (required)\n");
    printf("  -i <network interface>   Network interface to use (optional)\n");
    printf("  -t <update interval>     Update interval in seconds (optional)\n");
    printf("  -t <system uuid>         System UUID (optional)\n");
    printf("  -h                       Display this help message\n");
}

void parse_cli_arguments(int argc, char** argv) {
    cli_args.server_url       = NULL;
    cli_args.interface        = NULL;
    cli_args.uuid             = NULL;
    cli_args.heartbeat_period = 60;

    int32_t opt;
    while ((opt = getopt(argc, argv, "u:i:t:s:")) != -1) {
        switch (opt) {
            case 'u':
                cli_args.server_url = optarg;
                break;

            case 'i':
                cli_args.interface = optarg;
                break;

            case 't':
                cli_args.heartbeat_period = atoi(optarg);

                if (cli_args.heartbeat_period <= 0) {
                    printf("Invalid heartbeat update period parameter, setting to default.\n");
                    cli_args.heartbeat_period = 60;
                }

                break;

            case 's':
                cli_args.uuid = optarg;
                break;

            case 'h':
                print_usage(argv[0], WM_TRUE);
                exit(EXIT_SUCCESS);
            default:
                print_usage(argv[0], WM_TRUE);
                exit(EXIT_FAILURE);
        }
    }

    if (!cli_args.server_url) {
        print_usage(argv[0], WM_FALSE);
        exit(EXIT_FAILURE);
    }
}