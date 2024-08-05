#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "platform/ident.h"
#include "platform/device.h"
#include "utils/file_utils.h"
#include "network/request.h"
#include "network/http.h"

int main(int argc, char** argv) {
    (void)argc;
    (void)argv;

    http_header headers[] = {
        {.key = "Connection", .value = "close"},
    };

    http_request r;
    memset(&r, 0, sizeof(http_request));

    r.headers = headers;
    r.hlen    = ARRAY_SIZE(headers);
    r.url     = "http://192.168.2.19:8000";

    http_response* res = fetch(r);

    printf("Response status: %d\n", res->status);

    for (size_t i = 0; i < res->hlen; ++i) {
        printf("|%s|: |%s|\n", res->headers[i].key, res->headers[i].value);
    }

    printf("Body:\n%s\n", res->body);

    free_http_response(res);

    return EXIT_SUCCESS;
}