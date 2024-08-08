#include <network/file_transfer.h>
#include <network/request.h>
#include <utils/file_utils.h>
#include <utils/str.h>

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <time.h>

#define BUFFER_SIZE 8192

boolean_t download(const char* hostname, int port, const char* file_path, const char* local_file, boolean_t tls) {
    request_handle* handle = NULL;

    if (!request_start(&handle, hostname, port, tls)) {
        return WM_FALSE;
    }

    char request[BUFFER_SIZE];
    memset(request, 0, sizeof(request));

    snprintf(request, sizeof(request), "GET %s HTTP/1.1\r\nHost: %s\r\nConnection: close\r\n\r\n", file_path, hostname);

    if (request_write(handle, (uint8_t*)request, strlen(request)) < 0) {
        request_end(handle);
        return WM_FALSE;
    }

    FILE* file = fopen(local_file, "wb");
    if (!file) {
        request_end(handle);
        return WM_FALSE;
    }

    uint8_t   buffer[BUFFER_SIZE];
    ssize_t   bytes_read;
    boolean_t header_parsed = WM_FALSE;

    while ((bytes_read = request_read(handle, buffer, BUFFER_SIZE)) > 0) {
        if (!header_parsed) {
            char* headers_end = strstr((char*)buffer, "\r\n\r\n");
            if (headers_end != NULL) {
                ptrdiff_t header_len = headers_end - (char*)buffer + 4;
                header_parsed        = WM_TRUE;
                fwrite(headers_end + 4, sizeof(buffer[0]), bytes_read - header_len, file);
            }
        } else {
            fwrite(buffer, sizeof(buffer[0]), bytes_read, file);
        }
    }

    fclose(file);
    request_end(handle);

    return bytes_read >= 0;
}

boolean_t upload(const char* hostname, int port, const char* path, const char* local_file, boolean_t tls) {
    char boundary[40];
    generate_random_string(boundary, sizeof(boundary));

    ssize_t file_bytes = file_size(local_file);
    if (file_bytes < 0) {
        return WM_FALSE;
    }

    char initial_headers[BUFFER_SIZE];
    snprintf(initial_headers, sizeof(initial_headers),
             "--%s\r\n"
             "Content-Disposition: form-data; name=\"file\"; filename=\"%s\"\r\n"
             "Content-Type: application/octet-stream\r\n\r\n",
             boundary, filename(local_file));

    char end_boundary[BUFFER_SIZE];
    snprintf(end_boundary, sizeof(end_boundary), "\r\n--%s--\r\n", boundary);

    size_t content_length = strlen(initial_headers) + file_bytes + strlen(end_boundary);

    char request[BUFFER_SIZE];
    snprintf(request, sizeof(request),
             "POST %s HTTP/1.1\r\n"
             "Host: %s\r\n"
             "Content-Type: multipart/form-data; boundary=%s\r\n"
             "Content-Length: %zd\r\n"
             "Connection: close\r\n\r\n",
             path, hostname, boundary, content_length);

    request_handle* handle = NULL;

    if (!request_start(&handle, hostname, port, tls)) {
        return WM_FALSE;
    }
    if (request_write(handle, (uint8_t*)request, strlen(request)) < 0) {
        request_end(handle);
        return WM_FALSE;
    }

    if (request_write(handle, (uint8_t*)initial_headers, strlen(initial_headers)) < 0) {
        request_end(handle);
        return WM_FALSE;
    }

    FILE* file = fopen(local_file, "rb");
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

    request_end(handle);
    return WM_TRUE;
}
