#include <server_api/notify_configuration_reload.h>
#include <network/http.h>
#include <utils/url.h>
#include <utils/net.h>

static const char* endpoint = "/history/apply";

boolean_t notify_configuration_reload(const char* server_url, platform_info* info) {
    char      hostname[256] = {0};
    int       port          = 0;
    boolean_t tls           = WM_FALSE;

    if (!parse_url(server_url, hostname, sizeof(hostname), NULL, 0, &port, &tls)) {
        return WM_FALSE;
    }

    http_header headers[] = {
        {.key = "Connection", .value = "close"},
        {.key = "Host", .value = hostname},
        {.key = "X-Wallmon-UUID", .value = info->uuid},
    };

    http_request request;
    request.method        = HTTP_METHOD_POST;
    request.body          = NULL;
    request.body_len      = 0;
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
