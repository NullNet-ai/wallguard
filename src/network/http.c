#include <network/http.h>
#include <network/request.h>

#include <utils/net.h>

#include <assert.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

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

static size_t calculate_request_size(http_request* request, const char* path) {
    const char* method_str  = http_method_to_str(request->method);
    size_t      total_bytes = strlen(method_str) + strlen(path) + 12;

    if (request->headers && request->headers_count > 0) {
        for (size_t i = 0; i < request->headers_count; i++) {
            total_bytes += strlen(request->headers[i].key) + strlen(request->headers[i].value) + 4;
        }
    }

    total_bytes += 2;

    if (request->body) {
        total_bytes += request->body_len;
    }

    return total_bytes;
}

static boolean_t construct_request(const char* path, http_request* request, uint8_t** data, size_t* len) {
    size_t   total_bytes  = calculate_request_size(request, path);
    uint8_t* request_data = malloc(total_bytes + 1);

    if (!request_data) {
        return WM_FALSE;
    }

    const char* method_str = http_method_to_str(request->method);
    sprintf((char*)request_data, "%s %s HTTP/1.1\r\n", method_str, path);
    for (size_t i = 0; i < request->headers_count; i++) {
        strcat((char*)request_data, request->headers[i].key);
        strcat((char*)request_data, ": ");
        strcat((char*)request_data, request->headers[i].value);
        strcat((char*)request_data, "\r\n");
    }
    strcat((char*)request_data, "\r\n");

    if (request->body && request->body_len > 0) {
        memcpy(request_data + strlen((char*)request_data), request->body, request->body_len);
    }

    *data = request_data;
    *len  = total_bytes;

    return WM_TRUE;
}

http_response* fetch(const char* hostname, const char* path, int port, boolean_t tls, http_request* request) {
    if(!hostname || !path || port <= 0) {
        return NULL;
    }

    uint8_t* request_data   = NULL;
    size_t   request_length = 0;
    if (!construct_request(path, request, &request_data, &request_length)) {
        return NULL;
    }

    request_handle* handle = NULL;
    if (!request_start(&handle, hostname, port, tls)) {
        free(request_data);
        return NULL;
    }

    if (!request_write(handle, request_data, request_length)) {
        request_end(handle);
        free(request_data);
        return NULL;
    }

    free(request_data);

    char*  response_data = NULL;
    size_t response_len  = 0;

    if (!read_response_full(handle, (uint8_t**)&response_data, &response_len, WM_TRUE)) {
        request_end(handle);
        return NULL;
    }

    http_response* response = parse_response(response_data, response_len);
    free(response_data);
    request_end(handle);
    return response;
}
