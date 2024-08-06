#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "network/tls.h"

#define BUFFER_SIZE 8192

int main(int argc, char** argv) {
    (void)argc;
    (void)argv;

    tls_handle* handle   = NULL;
    const char* hostname = "example.com";
    int         port     = 443;

    // Initialize TLS connection
    if (!tls_start(&handle, hostname, port)) {
        fprintf(stderr, "Failed to establish TLS connection\n");
        return EXIT_FAILURE;
    }

    // Create HTTP GET request
    const char* request = "GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";

    // Send HTTP GET request
    if (tls_write(handle, (uint8_t*)request, strlen(request)) < 0) {
        fprintf(stderr, "Failed to send HTTP GET request\n");
        tls_end(handle);
        return EXIT_FAILURE;
    }

    // Buffer to store the response
    uint8_t buffer[BUFFER_SIZE];
    memset(buffer, 0, BUFFER_SIZE);

    // Read the response
    ssize_t bytes_read;
    while ((bytes_read = tls_read(handle, buffer, BUFFER_SIZE - 1)) > 0) {
        printf("%s", buffer);
        memset(buffer, 0, BUFFER_SIZE);  // Clear buffer for next read
    }

    if (bytes_read < 0) {
        fprintf(stderr, "Error reading from socket\n");
    }

    // Close TLS connection
    if (!tls_end(handle)) {
        fprintf(stderr, "Failed to close TLS connection\n");
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}
