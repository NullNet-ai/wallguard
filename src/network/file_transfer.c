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
