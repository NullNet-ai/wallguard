#include <network/http.h>
#include <network/request.h>

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
    uint8_t* request_data = malloc(total_bytes);
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

static char* read_response_full(request_handle* handle) {
    static const size_t buffer_size = 4096;

    char* response = malloc(buffer_size);
    if (!response) {
        return NULL;
    }

    size_t total_len = 0;
    size_t capacity  = buffer_size;

    uint8_t buffer[buffer_size];
    while (WM_TRUE) {
        memset(buffer, 0, sizeof(buffer));

        ssize_t bytes = request_read(handle, buffer, sizeof(buffer) - 1);

        if (bytes <= 0) {
            break;
        }

        if (total_len + bytes >= capacity) {
            capacity *= 2;

            char* new_buff = realloc(response, capacity);

            if (new_buff == NULL) {
                free(response);
                return NULL;
            }

            response = new_buff;
        }

        memcpy(response + total_len, buffer, bytes);
        total_len += bytes;
    }

    response[total_len] = '\0';
    return response;
}

static boolean_t parse_response(char* raw_response, size_t response_len, http_response* response) {
    response->status_code   = 0;
    response->body          = NULL;
    response->body_len      = 0;
    response->headers       = NULL;
    response->headers_count = 0;

    char* header_end = strstr(raw_response, "\r\n\r\n");
    if (header_end == NULL) {
        return WM_FALSE;
    }

    char*  body_start = header_end + 4;
    size_t body_len   = response_len - (body_start - raw_response);

    char* status_line_end = strstr(raw_response, "\r\n");
    if (status_line_end == NULL) {
        return WM_FALSE;
    }

    char status_line[2400];
    strncpy(status_line, raw_response, status_line_end - raw_response);
    status_line[status_line_end - raw_response] = '\0';

    char* status_code_str = strchr(status_line, ' ') + 1;
    response->status_code = atoi(status_code_str);

    char* header_start      = status_line_end + 2;
    char* header_current    = header_start;
    response->headers       = malloc(sizeof(http_header) * 16);
    response->headers_count = 0;

    while (header_current < header_end) {
        char* colon = strchr(header_current, ':');
        if (colon == NULL) {
            break;
        }

        char* value_start = colon + 2;
        char* line_end    = strstr(header_current, "\r\n");
        if (line_end == NULL) {
            break;
        }

        size_t key_len   = colon - header_current;
        size_t value_len = line_end - value_start;

        response->headers[response->headers_count].key   = malloc(key_len + 1);
        response->headers[response->headers_count].value = malloc(value_len + 1);

        strncpy((char*)response->headers[response->headers_count].key, header_current, key_len);
        char* key_ptr    = (char*)response->headers[response->headers_count].key;
        key_ptr[key_len] = '\0';

        strncpy((char*)response->headers[response->headers_count].value, value_start, value_len);
        char* value_ptr      = (char*)response->headers[response->headers_count].value;
        value_ptr[value_len] = '\0';

        response->headers_count++;

        header_current = line_end + 2;

        if (response->headers_count % 16 == 0) {
            response->headers = realloc(response->headers, sizeof(http_header) * (response->headers_count + 16));
        }
    }

    response->body = malloc(body_len + 1);
    memcpy(response->body, body_start, body_len);

    response->body[body_len] = '\0';
    response->body_len       = body_len;

    return WM_TRUE;
}

http_response* fetch(const char* hostname, const char* path, int port, boolean_t tls, http_request* request) {
    assert(hostname != NULL);
    assert(path != NULL);
    assert(port > 0);

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
        free(request_data);
        return NULL;
    }

    free(request_data);

    char* response_data = read_response_full(handle);
    if (!response_data) {
        return NULL;
    }

    http_response* response = malloc(sizeof(http_response));
    if (!parse_response(response_data, strlen(response_data), response)) {
        free(response_data);
        free(response);
        return NULL;
    }

    free(response_data);
    return response;
}

void release_response(http_response* response) {
    if (response->body) {
        free(response->body);
        response->body = NULL;
    }

    if (response->headers) {
        for (size_t i = 0; i < response->headers_count; i++) {
            if (response->headers[i].key) {
                free((void*)response->headers[i].key);
                response->headers[i].key = NULL;
            }

            if (response->headers[i].value) {
                free((void*)response->headers[i].value);
                response->headers[i].value = NULL;
            }
        }

        free(response->headers);
        response->headers = NULL;
    }

    free(response);
}
