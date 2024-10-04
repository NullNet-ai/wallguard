#ifndef _CLI_ARGS_H_
#define _CLI_ARGS_H_

#include <stdint.h>

struct cli_args {
    const char* server_url;
    const char* interface;
    int32_t     heartbeat_period;
};

extern struct cli_args cli_args;

void parse_cli_arguments(int argc, char** argv);

#endif
