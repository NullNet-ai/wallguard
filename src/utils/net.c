#include <utils/net.h>

#include <stdlib.h>
#include <string.h>

http_response* parse_response(char* raw_response, size_t response_len) {
    http_response* response = malloc(sizeof(http_response));

    response->status_code   = 0;
    response->body          = NULL;
    response->body_len      = 0;
    response->headers       = NULL;
    response->headers_count = 0;

    char* header_end = strstr(raw_response, "\r\n\r\n");
    if (header_end == NULL) {
        free(response);
        return NULL;
    }

    char*  body_start = header_end + 4;
    size_t body_len   = response_len - (body_start - raw_response);

    char* status_line_end = strstr(raw_response, "\r\n");
    if (status_line_end == NULL) {
        free(response);
        return NULL;
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

boolean_t read_response_full(request_handle* handle, uint8_t** data, size_t* len, boolean_t terminate) {
    static const size_t buffer_size = 4096;

    uint8_t* response = malloc(buffer_size);
    if (!response) {
        return WM_FALSE;
    }

    size_t total_len = 0;
    size_t capacity  = buffer_size;

    uint8_t buffer[buffer_size];
    while (WM_TRUE) {
        memset(buffer, 0, sizeof(buffer));

        ssize_t bytes = request_read(handle, buffer, sizeof(buffer) - 1);

        if (bytes < 0) {
            free(response);
            return WM_FALSE;
        }

        if (bytes == 0) {
            break;
        }

        if (total_len + bytes >= capacity) {
            capacity *= 2;

            uint8_t* new_buff = realloc(response, capacity);

            if (new_buff == NULL) {
                free(response);
                return WM_FALSE;
            }

            response = new_buff;
        }

        memcpy(response + total_len, buffer, bytes);
        total_len += bytes;
    }

    if (terminate) {
        response[total_len] = '\0';
    }

    *data = response;
    *len  = total_len;

    return WM_TRUE;
}
