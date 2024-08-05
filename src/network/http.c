#include <network/http.h>
#include <network/request.h>
#include <utils/str.h>

#include <string.h>
#include <stdio.h>
#include <stdlib.h>

static const char* http_method_to_str(http_method method) {
    switch (method) {
        case HTTP_METHOD_GET:
            return "GET";
        case HTTP_METHOD_POST:
            return "POST";
        case HTTP_METHOD_DELETE:
            return "DELETE";
        case HTTP_METHOD_PUT:
            return "PUT";
        default:
            return "GET";
    }
}

static boolean_t parse_url(const char* url, char* hostname, char* path, int* port, boolean_t* use_tls) {
    const char* protocol_end = strstr(url, "://");
    if (protocol_end != NULL) {
        ptrdiff_t protocol_len = protocol_end - url;
        if (strncmp(url, "http", protocol_len) == 0) {
            *use_tls = WM_FALSE;
            *port    = 80;
        } else if (strncmp(url, "https", protocol_len) == 0) {
            *use_tls = WM_TRUE;
            *port    = 443;
        } else {
            // Protocol not supported
            return WM_FALSE;
        }
        url = protocol_end + 3;
    } else {
        // Default, if protocol was not specified explicitly
        *port    = 443;
        *use_tls = WM_TRUE;
    }

    const char* path_start = strchr(url, '/');
    if (!path_start) {
        path_start = url + strlen(url);
        strcpy(path, "/");
    } else {
        strcpy(path, path_start);
    }

    const char* colon = strchr(url, ':');
    if (colon && colon < path_start) {
        *port = atoi(colon + 1);
        strncpy(hostname, url, colon - url);
        hostname[colon - url] = '\0';
    } else {
        strncpy(hostname, url, path_start - url);
        hostname[path_start - url] = '\0';
    }

    return WM_TRUE;
}

static size_t calculate_request_size(http_request* request, const char* path) {
    // METHOD + PATH + " HTTP/1.1\r\n"
    const char* method_str  = http_method_to_str(request->method);
    size_t      total_bytes = strlen(method_str) + strlen(path) + 12;

    if (request->headers && request->hlen > 0) {
        for (size_t i = 0; i < request->hlen; i++) {
            // "Key: Value\r\n"
            total_bytes += strlen(request->headers[i].key) + strlen(request->headers[i].value) + 4;
        }
    }
    // \r\n
    total_bytes += 2;

    if (request->body) {
        total_bytes += request->blen;
    }

    return total_bytes;
}

static uint8_t* perform_request(http_request* request) {
    char      hostname[256];
    char      path[256];
    int       port;
    boolean_t use_tls;

    if (!parse_url(request->url, hostname, path, &port, &use_tls)) {
        return NULL;
    }

    const char* method_str  = http_method_to_str(request->method);
    size_t      total_bytes = calculate_request_size(request, path);

    uint8_t* request_data = malloc(total_bytes);
    if (!request_data) {
        return NULL;
    }

    sprintf((char*)request_data, "%s %s HTTP/1.1\r\n", method_str, path);
    for (size_t i = 0; i < request->hlen; i++) {
        strcat((char*)request_data, request->headers[i].key);
        strcat((char*)request_data, ": ");
        strcat((char*)request_data, request->headers[i].value);
        strcat((char*)request_data, "\r\n");
    }
    strcat((char*)request_data, "\r\n");

    if (request->body) {
        memcpy(request_data + strlen((char*)request_data), request->body, request->blen);
    }

    uint8_t* response_data;
    if (use_tls) {
        response_data = request_tls(hostname, port, request_data, total_bytes);
    } else {
        response_data = request_tcp(hostname, port, request_data, total_bytes);
    }

    free(request_data);
    return response_data;
}

static boolean_t parse_response(http_response* response) {
    char* response_line = strstr((char*)response->__ptr, "\r\n");
    if (!response_line) {
        return WM_FALSE;
    }

    ptrdiff_t line_len        = response_line - (char*)response->__ptr;
    response->__ptr[line_len] = '\0';

    int http_major_ver;
    int http_minor_ver;
    sscanf((char*)response->__ptr, "HTTP/%d.%d %d", &http_major_ver, &http_minor_ver, &response->status);

    char* headers_begin = response_line + 2;
    char* headers_end   = strstr(headers_begin, "\r\n\r\n");

    if (!headers_begin || !headers_end) {
        return WM_FALSE;
    }

    ptrdiff_t headers_len      = headers_end - headers_begin;
    headers_begin[headers_len] = '\0';

    response->hlen    = count_substring_occurrences(headers_begin, "\r\n") + 1;
    response->headers = malloc(response->hlen * sizeof(http_header));

    for (size_t i = 0; i < response->hlen; ++i) {
        char*     header_end = strstr(headers_begin, "\r\n");
        ptrdiff_t header_line_len;
        if (header_end) {
            header_line_len = header_end - headers_begin;
        } else {
            header_line_len = headers_end - headers_begin;
        }

        response->headers[i].key = headers_begin;
        char* key_end            = strstr(headers_begin, ": ");
        *key_end                 = '\0';
        key_end += 2;

        response->headers[i].value = key_end;

        if (header_end) {
            *header_end = '\0';
        }

        headers_begin += header_line_len + 2;
    }

    response->body = (uint8_t*)headers_end + 4;
    response->blen = strlen((char*)response->body);

    return WM_TRUE;
}

http_response* fetch(http_request request) {
    uint8_t* data = perform_request(&request);
    if (!data) {
        return NULL;
    }

    http_response* response = malloc(sizeof(http_response));
    response->__ptr         = data;

    if (!parse_response(response)) {
        free_http_response(response);
    }

    return response;
}

void free_http_response(http_response* response) {
    if (response->headers) {
        free(response->headers);
    }

    if (response->__ptr) {
        free(response->__ptr);
    }

    free(response);
}

// @TODO: Ensure hostname and path buffers are large enough to handle the strings being copied into them.
// @TODO: Using strncpy and strcat without ensuring null termination can lead to buffer overflows or undefined behavior.
// @TODO: The function parse_response has no way to handle partial or malformed HTTP responses.
// @TODO: Ensure that all allocated memory is freed in case of an error to avoid memory leaks.
// @TODO: Using sprintf and strcat can be risky due to buffer overflows. Use safer alternatives like snprintf and strncat.