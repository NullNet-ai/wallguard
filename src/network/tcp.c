#include <network/tcp.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>

struct tcp_handle {
    int socket;
};

boolean_t tcp_start(tcp_handle** handle, const char* hostname, int port) {
    if (!hostname || port < 0) {
        return WM_FALSE;
    }

    int sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) {
        return WM_FALSE;
    }

    struct hostent* server = gethostbyname(hostname);
    if (server == NULL) {
        close(sockfd);
        return WM_FALSE;
    }

    struct sockaddr_in serv_addr;
    memset(&serv_addr, 0, sizeof(serv_addr));
    serv_addr.sin_family = AF_INET;
    memcpy(&serv_addr.sin_addr.s_addr, server->h_addr_list[0], server->h_length);
    serv_addr.sin_port = htons(port);

    if (connect(sockfd, (struct sockaddr*)&serv_addr, sizeof(serv_addr)) < 0) {
        close(sockfd);
        return WM_FALSE;
    }

    (*handle)         = malloc(sizeof(tcp_handle));
    (*handle)->socket = sockfd;
    return WM_TRUE;
}

ssize_t tcp_write(tcp_handle* handle, uint8_t* data, size_t len) {
    if (!handle || !data || len == 0) {
        return -1;
    }

    return write(handle->socket, data, len);
}

ssize_t tcp_read(tcp_handle* handle, uint8_t* data, size_t len) {
    if (!handle || !data || len == 0) {
        return -1;
    }

    return read(handle->socket, data, len);
}

void tcp_end(tcp_handle* handle) {
    close(handle->socket);
    free(handle);
}
