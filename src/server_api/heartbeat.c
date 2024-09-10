#include <logger/logger.h>
#include <server_api/heartbeat.h>
#include <utils/url.h>
#include <utils/net.h>
#include <utils/str.h>
#include <network/http.h>

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

static void parse_action(http_response* response, server_action* action) {
    if (!response || !response->body || response->body_len == 0) {
        *action = SACTION_NONE;
        return;
    }

    long retval;
    if (!string_to_integer(response->body, &retval, 10)) {
        WLOG_ERROR("Failed to parse response body as a number. Setting action to None. Body: %s", response->body);
        *action = SACTION_NONE;
        return;
    }

    switch (retval) {
        case 0:
            *action = SACTION_REUPLOAD;
            break;
        default:
            WLOG_ERROR("Unsupported action %d, setting to None", retval);
            *action = SACTION_NONE;
            break;
    }
}

static const char* endpoint = "/wallmon/heartbeat";

boolean_t heartbeat_request(const char* server_url, platform_info* info, server_action* action) {
    char      hostname[256] = {0};
    int       port          = 0;
    boolean_t tls           = WM_FALSE;

    if (!parse_url(server_url, hostname, sizeof(hostname), NULL, 0, &port, &tls)) {
        WLOG_ERROR("Failed to parse URL: %s", server_url);
        return WM_FALSE;
    }

    char buffer[64];
    memset(buffer, 0, sizeof(buffer));
    snprintf(buffer, sizeof(buffer) - 1, "{\"uuid\":\"%s\"}", info->uuid);

    size_t body_len = strlen(buffer);

    char buf[32];
    snprintf(buf, sizeof(buf) - 1, "%ld", body_len);

    http_header headers[] = {
        {.key = "Content-Type", .value = "application/json"},
        {.key = "Accept", .value = "application/json"},
        {.key = "Connection", .value = "close"},
        {.key = "Content-Length", .value = buf},
        {.key = "Host", .value = hostname},
    };

    http_request request;
    request.method        = HTTP_METHOD_POST;
    request.body          = buffer;
    request.body_len      = body_len;
    request.headers       = headers;
    request.headers_count = ARRAY_SIZE(headers);

    http_response* response = fetch(hostname, endpoint, port, tls, &request);
    if (!response) {
        return WM_FALSE;
    }

    boolean_t success = response->status_code >= 200 && response->status_code < 300;

    if (success) {
        parse_action(response, action);
    }

    release_response(response);
    return success;
}
