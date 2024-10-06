#ifndef _CLI_ARGS_H_
#define _CLI_ARGS_H_

#include <stdint.h>

struct cli_args {
    const char* server_url;
    const char* interface;
    const char* uuid;
    int32_t     heartbeat_period;
};

extern struct cli_args cli_args;

/**
 * @brief Parses command-line arguments and populates the `cli_args` structure.
 * 
 * @param argc The number of command-line arguments passed to the program.
 * @param argv The array of command-line arguments (strings).
 */
void parse_cli_arguments(int argc, char** argv);

#endif
