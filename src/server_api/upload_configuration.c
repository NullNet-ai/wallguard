#include <server_api/upload_configuration.h>
#include <network/request.h>
#include <utils/url.h>
#include <utils/net.h>
#include <utils/file_utils.h>
#include <utils/str.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define BUFFER_SIZE 4096

static const char* endpoint = "/firewall/configuration";

boolean_t upload_configuration(const char* server_url, const char* path, platform_info* info) {
    ssize_t file_bytes = file_size(path);
    if (file_bytes <= 0) {
        return WM_FALSE;
    }

    char      hostname[256];
    int       port = 0;
    boolean_t tls  = WM_FALSE;

    if (!parse_url(server_url, hostname, sizeof(hostname), NULL, 0, &port, &tls)) {
        return WM_FALSE;
    }

    request_handle* handle = NULL;

    if (!request_start(&handle, hostname, port, tls)) {
        return WM_FALSE;
    }

    char boundary[40] = {0};
    generate_random_string(boundary, sizeof(boundary));

    char initial_headers[BUFFER_SIZE];
    snprintf(initial_headers, sizeof(initial_headers),
             "--%s\r\n"
             "Content-Disposition: form-data; name=\"file\"; filename=\"%s\"\r\n"
             "Content-Type: application/octet-stream\r\n\r\n",
             boundary, filename(path));

    char end_boundary[BUFFER_SIZE] = {0};
    snprintf(end_boundary, sizeof(end_boundary), "\r\n--%s--\r\n", boundary);

    size_t content_length = strlen(initial_headers) + file_bytes + strlen(end_boundary);

    char request[BUFFER_SIZE];
    snprintf(request, sizeof(request),
             "POST %s HTTP/1.1\r\n"
             "Host: %s\r\n"
             "Content-Type: multipart/form-data; boundary=%s\r\n"
             "Content-Length: %zd\r\n"
             "X-Wallmon-UUID: %s\r\n"
             "Connection: close\r\n\r\n",
             endpoint, hostname, boundary, content_length, info->uuid);

    if (request_write(handle, (uint8_t*)request, strlen(request)) < 0) {
        request_end(handle);
        return WM_FALSE;
    }

    if (request_write(handle, (uint8_t*)initial_headers, strlen(initial_headers)) < 0) {
        request_end(handle);
        return WM_FALSE;
    }

    FILE* file = fopen(path, "rb");
    if (!file) {
        request_end(handle);
        return WM_FALSE;
    }

    uint8_t buffer[BUFFER_SIZE];
    ssize_t bytes_read;

    while ((bytes_read = fread(buffer, 1, BUFFER_SIZE, file)) > 0) {
        if (request_write(handle, buffer, bytes_read) < 0) {
            fclose(file);
            request_end(handle);
            return WM_FALSE;
        }
    }

    fclose(file);

    if (request_write(handle, (uint8_t*)end_boundary, strlen(end_boundary)) < 0) {
        request_end(handle);
        return WM_FALSE;
    }

    char*  response_data   = NULL;
    size_t response_length = 0;
    if (!read_response_full(handle, (uint8_t**)&response_data, &response_length, WM_TRUE)) {
        return WM_FALSE;
    }

    request_end(handle);

    http_response* response = parse_response(response_data, response_length);
    free(response_data);

    if (!response) {
        return WM_FALSE;
    }

    boolean_t retval = response->status_code >= 200 && response->status_code <= 299;
    release_response(response);

    return retval;
}