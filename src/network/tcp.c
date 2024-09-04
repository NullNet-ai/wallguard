#include <network/tcp.h>
#include <logger/logger.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <errno.h>

struct tcp_handle {
    int socket;
};

static inline void log_socket_error(const char* message) { WLOG_ERROR("%s: %s", message, strerror(errno)); }

boolean_t tcp_start(tcp_handle** handle, const char* hostname, int port) {
    if (!hostname || port < 0) {
        WLOG_ERROR("tcp_start: invalid arguments.");
        return WM_FALSE;
    }

    int sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) {
        log_socket_error("Failed to create socket");
        return WM_FALSE;
    }

    struct hostent* server = gethostbyname(hostname);
    if (server == NULL) {
        log_socket_error("Failed to resolve hostname");
        close(sockfd);
        return WM_FALSE;
    }

    struct sockaddr_in serv_addr;
    memset(&serv_addr, 0, sizeof(serv_addr));
    serv_addr.sin_family = AF_INET;
    memcpy(&serv_addr.sin_addr.s_addr, server->h_addr_list[0], server->h_length);
    serv_addr.sin_port = htons(port);

    if (connect(sockfd, (struct sockaddr*)&serv_addr, sizeof(serv_addr)) < 0) {
        log_socket_error("Failed to connect to server");
        close(sockfd);
        return WM_FALSE;
    }

    (*handle)         = malloc(sizeof(tcp_handle));
    (*handle)->socket = sockfd;
    return WM_TRUE;
}

ssize_t tcp_write(tcp_handle* handle, uint8_t* data, size_t len) {
    if (!handle || !data || len == 0) {
        WLOG_ERROR("tcp_write: invalid arguments.");
        return -1;
    }

    ssize_t result = write(handle->socket, data, len);

    if (result < 0) {
        log_socket_error("Failed to write data to socket");
    }

    return result;
}

ssize_t tcp_read(tcp_handle* handle, uint8_t* data, size_t len) {
    if (!handle || !data || len == 0) {
        WLOG_ERROR("tcp_read: invalid arguments.");
        return -1;
    }

    ssize_t result = read(handle->socket, data, len);

    if (result < 0) {
        log_socket_error("Failed to read data from socket");
    }

    return result;
}

void tcp_end(tcp_handle* handle) {
    close(handle->socket);
    free(handle);
}
