#include <logger/logger.h>
#include <server_api/request_registration.h>
#include <network/http.h>
#include <utils/url.h>
#include <utils/net.h>

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <assert.h>

static const char* endpoint      = "/wallmon/registration";
static const char* body_template = "{\"uuid\":\"%s\",\"make\":\"%s\",\"version\":\"%s\"}";

boolean_t request_registration(const char* server_url, platform_info* info) {
    char      hostname[256] = {0};
    int       port          = 0;
    boolean_t tls           = WM_FALSE;

    if (!parse_url(server_url, hostname, sizeof(hostname), NULL, 0, &port, &tls)) {
        WLOG_ERROR("Failed to parse URL: %s", server_url);
        return WM_FALSE;
    }

    char body[1024];
    memset(body, 0, sizeof(body));
    snprintf(body, sizeof(body) - 1, body_template, info->uuid, info->model, info->version);

    char buf[32];
    snprintf(buf, sizeof(buf) - 1, "%ld", strlen(body));

    http_header headers[] = {
        {.key = "Content-Type", .value = "application/json"},
        {.key = "Accept", .value = "application/json"},
        {.key = "Connection", .value = "close"},
        {.key = "Content-Length", .value = buf},
        {.key = "Host", .value = hostname},
    };

    http_request request;
    request.method        = HTTP_METHOD_POST;
    request.body          = body;
    request.body_len      = strlen(body);
    request.headers       = headers;
    request.headers_count = ARRAY_SIZE(headers);

    http_response* response = fetch(hostname, endpoint, port, tls, &request);
    if (!response) {
        return WM_FALSE;
    }

    boolean_t success = response->status_code >= 200 && response->status_code < 300;
    release_response(response);
    return success;
}
