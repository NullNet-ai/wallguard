#include "server_requests.h"
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

static inline boolean_t util_is_status_ok(int status) { return status >= 200 && status < 299; }

boolean_t wallmom_registration(const char* server_url, platform_info* info) {
    CURL* curl = curl_easy_init();
    if (!curl) {
        WLOG_ERROR("Failed to initialize CURL");
        return WM_FALSE;
    }

    curl_easy_setopt(curl, CURLOPT_POST, 1);

    if (!util_curl_set_url(curl, server_url, "/wallmon/registration")) {
        WLOG_WARN("URL: %s is too long.");
    }

    char body[1024];
    memset(body, 0, sizeof(body));
    snprintf(body, sizeof(body), "{\"uuid\":\"%s\",\"make\":\"%s\",\"version\":\"%s\"}", info->uuid, info->model,
             info->version);

    char*              hvalues[] = {"Content-Type: application/json"};
    struct curl_slist* headers   = util_curl_set_headers(curl, hvalues, ARRAY_SIZE(hvalues));

    CURLcode code = curl_easy_perform(curl);

    int http_status = 0;
    if (code == CURLE_OK) {
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_status);
    } else {
        WLOG_ERROR("Request failed. %s", curl_easy_strerror(code));
    }

    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);

    return util_is_status_ok(http_status);
}

boolean_t wallmon_heartbeat(const char* server_url, platform_info* info) {
    CURL* curl = curl_easy_init();
    if (!curl) {
        WLOG_ERROR("Failed to initialize CURL");
        return WM_FALSE;
    }

    curl_easy_setopt(curl, CURLOPT_POST, 1);

    if (!util_curl_set_url(curl, server_url, "/wallmon/heartbeat")) {
        WLOG_WARN("URL: %s is too long.");
    }

    char body_buffer[64] = {0};
    snprintf(body_buffer, sizeof(body_buffer) - 1, "{\"uuid\":\"%s\"}", info->uuid);
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, body_buffer);

    char*              hvalues[] = {"Content-Type: application/json"};
    struct curl_slist* headers   = util_curl_set_headers(curl, hvalues, ARRAY_SIZE(hvalues));

    CURLcode code = curl_easy_perform(curl);

    int http_status = 0;
    if (code == CURLE_OK) {
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_status);
    } else {
        WLOG_ERROR("Request failed. %s", curl_easy_strerror(code));
    }

    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);

    return util_is_status_ok(http_status);
}

boolean_t wallmon_uploadcfg(const char* server_url, const char* path, platform_info* info, boolean_t applied) {
    CURL* curl = curl_easy_init();
    if (!curl) {
        WLOG_ERROR("Failed to initialize CURL");
        return WM_FALSE;
    }

    if (!util_curl_set_url(curl, server_url, "/wallmon/cfg/upload")) {
        WLOG_WARN("URL: %s is too long.");
    }

    struct curl_httppost* formpost = NULL;
    struct curl_httppost* lastptr  = NULL;

    curl_formadd(&formpost, &lastptr, CURLFORM_COPYNAME, "file", CURLFORM_FILE, path, CURLFORM_END);
    curl_formadd(&formpost, &lastptr, CURLFORM_COPYNAME, "uuid", CURLFORM_PTRCONTENTS, info->uuid, CURLFORM_END);

    char buffer[2] = {0};
    snprintf(buffer, sizeof(buffer), "%d", applied ? 1 : 0);
    curl_formadd(&formpost, &lastptr, CURLFORM_COPYNAME, "applied", CURLFORM_PTRCONTENTS, buffer, CURLFORM_END);

    curl_easy_setopt(curl, CURLOPT_HTTPPOST, formpost);

    CURLcode code = curl_easy_perform(curl);

    int http_status = 0;
    if (code == CURLE_OK) {
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_status);
    } else {
        WLOG_ERROR("Request failed. %s", curl_easy_strerror(code));
    }

    curl_formfree(formpost);
    curl_easy_cleanup(curl);

    return util_is_status_ok(http_status);
}
