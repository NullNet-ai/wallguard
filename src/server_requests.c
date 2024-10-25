#include "server_requests.h"
#include "config.h"
#include <curl/curl.h>
#include <logger/logger.h>
#include <string.h>

static inline struct curl_slist* util_curl_set_headers(CURL* curl, char** headers, size_t len) {
    struct curl_slist* headers_list = NULL;

    for (size_t i = 0; i < len; ++i) {
        headers_list = curl_slist_append(headers_list, headers[i]);
    }

    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers_list);
    return headers_list;
}

static inline boolean_t util_curl_set_url(CURL* curl, const char* hostname, const char* endpoint) {
    char buffer[128] = {0};

    int total = snprintf(buffer, sizeof(buffer), "%s%s", hostname, endpoint);
    curl_easy_setopt(curl, CURLOPT_URL, buffer);

    return total >= 0 && (size_t)total < sizeof(buffer);
}

static int curl_perform_request(CURL* curl) {
    // @TODO
    // if (cli_args.interface) {
    //     curl_easy_setopt(curl, CURLOPT_INTERFACE, cli_args.interface);
    // }

    // @TODO
    // No self-signed certificates
    // Certificates verifications

    CURLcode code = curl_easy_perform(curl);

    int http_status = -1;

    if (code == CURLE_OK) {
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_status);
    } else {
        WALLMON_LOG_ERROR("Request failed. %s", curl_easy_strerror(code));
    }

    return http_status;
}

static inline boolean_t util_is_status_ok(int status) { return status >= 200 && status < 299; }

boolean_t wallmom_registration(platform_info* info) {
    CURL* curl = curl_easy_init();
    if (!curl) {
        WALLMON_LOG_ERROR("Failed to initialize CURL");
        return WM_FALSE;
    }

    curl_easy_setopt(curl, CURLOPT_POST, 1);

    if (!util_curl_set_url(curl, cfg_get_server_url(), "/wallmon/registration")) {
        WALLMON_LOG_ERROR("URL: %s is too long.");
        curl_easy_cleanup(curl);
        return WM_FALSE;
    }

    char body[1024] = {0};
    char* template  = "{\"uuid\":\"%s\",\"make\":\"%s\",\"version\":\"%s\"}";
    snprintf(body, sizeof(body), template, info->uuid, info->model, info->version);
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, body);

    char*              hvalues[] = {"Content-Type: application/json"};
    struct curl_slist* headers   = util_curl_set_headers(curl, hvalues, ARRAY_SIZE(hvalues));

    int http_status = curl_perform_request(curl);

    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);

    return util_is_status_ok(http_status);
}

boolean_t wallmon_heartbeat(platform_info* info) {
    CURL* curl = curl_easy_init();
    if (!curl) {
        WALLMON_LOG_ERROR("Failed to initialize CURL");
        return WM_FALSE;
    }

    curl_easy_setopt(curl, CURLOPT_POST, 1);

    if (!util_curl_set_url(curl, cfg_get_server_url(), "/wallmon/heartbeat")) {
        WALLMON_LOG_ERROR("URL: %s is too long.");
        curl_easy_cleanup(curl);
        return WM_FALSE;
    }

    char body_buffer[64] = {0};
    snprintf(body_buffer, sizeof(body_buffer) - 1, "{\"uuid\":\"%s\"}", info->uuid);
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, body_buffer);

    char*              hvalues[] = {"Content-Type: application/json"};
    struct curl_slist* headers   = util_curl_set_headers(curl, hvalues, ARRAY_SIZE(hvalues));

    int http_status = curl_perform_request(curl);

    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);

    return util_is_status_ok(http_status);
}

boolean_t wallmon_uploadcfg(const char* path, platform_info* info, boolean_t applied) {
    CURL* curl = curl_easy_init();
    if (!curl) {
        WALLMON_LOG_ERROR("Failed to initialize CURL");
        return WM_FALSE;
    }

    if (!util_curl_set_url(curl, cfg_get_server_url(), "/wallmon/cfg/upload")) {
        WALLMON_LOG_ERROR("URL: %s is too long.");
        curl_easy_cleanup(curl);
        return WM_FALSE;
    }

    struct curl_mime* multipart = curl_mime_init(curl);
    if (!multipart) {
        WALLMON_LOG_ERROR("Failed to initialize curl mime");
        curl_easy_cleanup(curl);
        return WM_FALSE;
    }

    struct curl_mimepart* part = curl_mime_addpart(multipart);
    curl_mime_name(part, "file");
    curl_mime_filedata(part, path);

    part = curl_mime_addpart(multipart);
    curl_mime_name(part, "uuid");
    curl_mime_data(part, info->uuid, CURL_ZERO_TERMINATED);

    char buffer[2] = {0};
    snprintf(buffer, sizeof(buffer), "%d", applied ? 1 : 0);

    part = curl_mime_addpart(multipart);
    curl_mime_name(part, "applied");
    curl_mime_data(part, buffer, CURL_ZERO_TERMINATED);

    curl_easy_setopt(curl, CURLOPT_MIMEPOST, multipart);

    int http_status = curl_perform_request(curl);

    curl_easy_cleanup(curl);

    return util_is_status_ok(http_status);
}
