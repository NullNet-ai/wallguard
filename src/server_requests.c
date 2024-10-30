#include "server_requests.h"
#include "config.h"
#include <curl/curl.h>
#include <logger/logger.h>
#include <string.h>
#include <stdlib.h>

// [UTILITIES] Begin ------------------------------------------------------------------------------------------------ //
#define UTIL_DEFINE_AUTH_HEADER(header_name, session_token)                                                         \
    char auth_header[1024] = {0};                                                                                   \
    int  result            = snprintf(auth_header, sizeof(auth_header), "Authorization: Bearer %s", session_token); \
    if (result < 0 || (size_t)result >= sizeof(auth_header)) {                                                      \
        WALLMON_LOG_ERROR("Failed to create auth header");                                                          \
        return WM_FALSE;                                                                                            \
    }
#define _UTIL_CURL_INIT(curl)                           \
    CURL* curl = curl_easy_init();                      \
    if (!curl) {                                        \
        WALLMON_LOG_ERROR("Failed to initialize CURL"); \
        return WM_FALSE;                                \
    }

#define _UTIL_CURL_SET_URL(curl, endpoint)         \
    if (!util_curl_set_url(curl, endpoint)) {      \
        WALLMON_LOG_ERROR("URL: %s is too long."); \
        curl_easy_cleanup(curl);                   \
        return WM_FALSE;                           \
    }

#define UTIL_CURL_INIT(curl, endpoint) \
    _UTIL_CURL_INIT(curl);             \
    _UTIL_CURL_SET_URL(curl, endpoint);

#define UTIL_CURL_INIT_MIME(curl, multipart)                 \
    struct curl_mime* multipart = curl_mime_init(curl);      \
    if (!multipart) {                                        \
        WALLMON_LOG_ERROR("Failed to initialize curl mime"); \
        curl_easy_cleanup(curl);                             \
        return WM_FALSE;                                     \
    }

static inline struct curl_slist* util_curl_set_headers(CURL* curl, char** headers, size_t len) {
    struct curl_slist* headers_list = NULL;

    for (size_t i = 0; i < len; ++i) {
        headers_list = curl_slist_append(headers_list, headers[i]);
    }

    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers_list);
    return headers_list;
}

static inline boolean_t util_curl_set_url(CURL* curl, const char* endpoint) {
    const char* hostname    = cfg_get_server_url();
    char        buffer[128] = {0};
    int         total       = snprintf(buffer, sizeof(buffer), "%s%s", hostname, endpoint);
    curl_easy_setopt(curl, CURLOPT_URL, buffer);

    return total >= 0 && (size_t)total < sizeof(buffer);
}

static int curl_perform_request(CURL* curl) {
    const char* ifname = cfg_get_netwrok_interface();
    if (ifname) {
        curl_easy_setopt(curl, CURLOPT_INTERFACE, ifname);
    }

    curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 1L);
    curl_easy_setopt(curl, CURLOPT_SSL_VERIFYHOST, 2L);
    curl_easy_setopt(curl, CURLOPT_TIMEOUT, 10L);

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

// [UTILITIES] End -------------------------------------------------------------------------------------------------- //

struct memstruct {
    char*  memory;
    size_t size;
    size_t allocated;
};

static size_t write_mem_cb(void* contents, size_t size, size_t nmemb, void* userp) {
    size_t            realsize = size * nmemb;
    struct memstruct* mem      = (struct memstruct*)userp;

    if (mem->size + realsize >= mem->allocated) {
        size_t new_size = mem->size + realsize + 1;
        mem->memory     = W_REALLOC(mem->memory, new_size);
        mem->allocated  = new_size;
    }

    memcpy(&(mem->memory[mem->size]), contents, realsize);
    mem->size += realsize;
    mem->memory[mem->size] = 0;
    return realsize;
}

boolean_t wallmon_authenticate(char** session_token) {
    UTIL_CURL_INIT(curl, "/wallmon/authenticate");
    UTIL_CURL_INIT_MIME(curl, multipart);
    curl_easy_setopt(curl, CURLOPT_POST, 1);

    struct curl_mimepart* part = curl_mime_addpart(multipart);
    curl_mime_name(part, "api_key");
    curl_mime_data(part, cfg_get_api_key(), CURL_ZERO_TERMINATED);

    part = curl_mime_addpart(multipart);
    curl_mime_name(part, "access_token");
    curl_mime_data(part, cfg_get_access_token(), CURL_ZERO_TERMINATED);

    curl_easy_setopt(curl, CURLOPT_MIMEPOST, multipart);

    struct memstruct chunk;
    chunk.memory    = W_CALLOC(64, sizeof(char));
    chunk.allocated = 64;
    chunk.size      = 0;

    curl_easy_setopt(curl, CURLOPT_WRITEDATA, (void*)&chunk);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_mem_cb);

    int http_status = curl_perform_request(curl);

    curl_mime_free(multipart);
    curl_easy_cleanup(curl);

    if (util_is_status_ok(http_status)) {
        *session_token = chunk.memory;
        return WM_TRUE;
    } else {
        free(chunk.memory);
        return WM_FALSE;
    }
}

boolean_t wallmon_setup(const char* session_token, const platform_info* info) {
    UTIL_DEFINE_AUTH_HEADER(auth_header, session_token);
    UTIL_CURL_INIT(curl, "/wallmon/setup");
    curl_easy_setopt(curl, CURLOPT_POST, 1);

    UTIL_CURL_INIT_MIME(curl, multipart);

    struct curl_mimepart* part = curl_mime_addpart(multipart);
    curl_mime_name(part, "uuid");
    curl_mime_data(part, info->uuid, CURL_ZERO_TERMINATED);

    part = curl_mime_addpart(multipart);
    curl_mime_name(part, "model");
    curl_mime_data(part, info->model, CURL_ZERO_TERMINATED);

    part = curl_mime_addpart(multipart);
    curl_mime_name(part, "version");
    curl_mime_data(part, info->version, CURL_ZERO_TERMINATED);

    part = curl_mime_addpart(multipart);
    curl_mime_name(part, "name");
    curl_mime_data(part, info->instance_name, CURL_ZERO_TERMINATED);

    curl_easy_setopt(curl, CURLOPT_MIMEPOST, multipart);

    char*              hvalues[] = {auth_header};
    struct curl_slist* headers   = util_curl_set_headers(curl, hvalues, ARRAY_SIZE(hvalues));

    int http_status = curl_perform_request(curl);

    curl_slist_free_all(headers);
    curl_mime_free(multipart);
    curl_easy_cleanup(curl);

    return util_is_status_ok(http_status);
}

boolean_t wallmon_fetch_key(const char* session_token, char** public_key) {
    UTIL_DEFINE_AUTH_HEADER(auth_header, session_token);
    UTIL_CURL_INIT(curl, "/wallmon/public_key");

    char*              hvalues[] = {auth_header};
    struct curl_slist* headers   = util_curl_set_headers(curl, hvalues, ARRAY_SIZE(hvalues));

    struct memstruct chunk;
    chunk.memory    = W_CALLOC(64, sizeof(char));
    chunk.allocated = 64;
    chunk.size      = 0;

    curl_easy_setopt(curl, CURLOPT_WRITEDATA, (void*)&chunk);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_mem_cb);

    int http_status = curl_perform_request(curl);

    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);

    if (util_is_status_ok(http_status)) {
        *public_key = chunk.memory;
        return WM_TRUE;
    } else {
        free(chunk.memory);
        return WM_FALSE;
    }
}

boolean_t wallmon_upload_configuration(const char* session_token, const char* path, const char* key, const char* iv,
                                       const platform_info* info) {
    UTIL_DEFINE_AUTH_HEADER(auth_header, session_token);
    UTIL_CURL_INIT(curl, "/wallmon/cfg/upload");
    curl_easy_setopt(curl, CURLOPT_POST, 1);

    UTIL_CURL_INIT_MIME(curl, multipart);

    struct curl_mimepart* part = curl_mime_addpart(multipart);
    curl_mime_name(part, "key");
    curl_mime_data(part, key, CURL_ZERO_TERMINATED);

    part = curl_mime_addpart(multipart);
    curl_mime_name(part, "iv");
    curl_mime_data(part, iv, CURL_ZERO_TERMINATED);

    part = curl_mime_addpart(multipart);
    curl_mime_name(part, "applied");
    if (info->dirty) {
        curl_mime_data(part, "0", CURL_ZERO_TERMINATED);
    } else {
        curl_mime_data(part, "1", CURL_ZERO_TERMINATED);
    }

    part = curl_mime_addpart(multipart);
    curl_mime_name(part, "file");
    curl_mime_filedata(part, path);

    curl_easy_setopt(curl, CURLOPT_MIMEPOST, multipart);

    char*              hvalues[] = {auth_header};
    struct curl_slist* headers   = util_curl_set_headers(curl, hvalues, ARRAY_SIZE(hvalues));

    int http_status = curl_perform_request(curl);

    curl_slist_free_all(headers);
    curl_mime_free(multipart);
    curl_easy_cleanup(curl);
    return util_is_status_ok(http_status);
}

boolean_t wallmon_heartbeat(const char* session_token) {
    UTIL_DEFINE_AUTH_HEADER(auth_header, session_token);
    UTIL_CURL_INIT(curl, "/wallmon/heartbeat");
    curl_easy_setopt(curl, CURLOPT_POST, 1);
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, "");

    char*              hvalues[] = {auth_header};
    struct curl_slist* headers   = util_curl_set_headers(curl, hvalues, ARRAY_SIZE(hvalues));

    int http_status = curl_perform_request(curl);

    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);

    return util_is_status_ok(http_status);
}
