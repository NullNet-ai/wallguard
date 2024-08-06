#include <network/request.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>

#include <openssl/ssl.h>
#include <openssl/bio.h>
#include <openssl/err.h>

#define BUFFER_SIZE 8192

uint8_t* request_tls(const char* hostname, int port, uint8_t* data, size_t len) {
    if (hostname == NULL || port < 0 || data == NULL || len == 0) {
        return NULL;
    }

    SSL_CTX* ctx      = NULL;
    BIO*     bio      = NULL;
    SSL*     ssl      = NULL;
    uint8_t* response = NULL;

    SSL_library_init();
    SSL_load_error_strings();
    OpenSSL_add_all_algorithms();

    ctx = SSL_CTX_new(TLS_client_method());
    if (ctx == NULL) {
        ERR_print_errors_fp(stderr);
        goto __exit;
    }

    if (!SSL_CTX_set_min_proto_version(ctx, TLS1_2_VERSION)) {
        ERR_print_errors_fp(stderr);
        goto __exit;
    }

    bio = BIO_new_ssl_connect(ctx);
    if (bio == NULL) {
        ERR_print_errors_fp(stderr);
        goto __exit;
    }

    char conn_string[256];
    memset(conn_string, 0, sizeof(conn_string));
    snprintf(conn_string, sizeof(conn_string), "%s:%d", hostname, port);
    BIO_set_conn_hostname(bio, conn_string);

    BIO_get_ssl(bio, &ssl);
    if (ssl == NULL) {
        ERR_print_errors_fp(stderr);
        goto __exit;
    }

    SSL_set_mode(ssl, SSL_MODE_AUTO_RETRY);

    if (BIO_do_connect(bio) <= 0) {
        ERR_print_errors_fp(stderr);
        goto __exit;
    }

    if (BIO_do_handshake(bio) <= 0) {
        ERR_print_errors_fp(stderr);
        goto __exit;
    }

    if (BIO_write(bio, data, len) <= 0) {
        if (!BIO_should_retry(bio)) {
            ERR_print_errors_fp(stderr);
            goto __exit;
        }
    }

    size_t total_len = 0;
    size_t capacity  = BUFFER_SIZE;

    response = malloc(capacity);
    if (response == NULL) {
        goto __exit;
    }

    uint8_t buffer[BUFFER_SIZE];
    while (WM_TRUE) {
        memset(buffer, 0, sizeof(buffer));
        int bytes = BIO_read(bio, buffer, sizeof(buffer) - 1);

        if (bytes <= 0) {
            break;
        }

        if (total_len + bytes >= capacity) {
            capacity *= 2;

            uint8_t* r = realloc(response, capacity);
            if (r == NULL) {
                free(response);
                response = NULL;
                goto __exit;
            }

            response = r;
        }

        memcpy(response + total_len, buffer, bytes);
        total_len += bytes;
    }

    response[total_len] = '\0';

__exit:
    if (bio) {
        BIO_free_all(bio);
    }

    if (ctx) {
        SSL_CTX_free(ctx);
    }

    return response;
}

uint8_t* request_tcp(const char* hostname, int port, uint8_t* data, size_t len) {
    if (hostname == NULL || port < 0 || data == NULL || len == 0) {
        return NULL;
    }

    uint8_t* response = NULL;

    int sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) {
        fprintf(stderr, "Cannot open a socket\n");
        goto __exit;
    }

    struct hostent* server = gethostbyname(hostname);
    if (server == NULL) {
        fprintf(stderr, "ERROR, no such host\n");
        goto __exit;
    }

    struct sockaddr_in serv_addr;
    memset(&serv_addr, 0, sizeof(serv_addr));
    serv_addr.sin_family = AF_INET;
    memcpy(&serv_addr.sin_addr.s_addr, server->h_addr_list[0], server->h_length);
    serv_addr.sin_port = htons(port);

    if (connect(sockfd, (struct sockaddr*)&serv_addr, sizeof(serv_addr)) < 0) {
        fprintf(stderr, "ERROR connecting");
        goto __exit;
    }

    if (write(sockfd, data, len) < 0) {
        fprintf(stderr, "ERROR writing to socket");
        goto __exit;
    }

    size_t total_len = 0;
    size_t capacity  = BUFFER_SIZE;

    response = malloc(capacity);
    if (response == NULL) {
        goto __exit;
    }

    uint8_t buffer[BUFFER_SIZE];
    while (WM_TRUE) {
        memset(buffer, 0, sizeof(buffer));
        int bytes = read(sockfd, buffer, sizeof(buffer) - 1);

        if (bytes <= 0) {
            break;
        }

        if (total_len + bytes >= capacity) {
            capacity *= 2;

            uint8_t* r = realloc(response, capacity);
            if (r == NULL) {
                free(response);
                response = NULL;
                goto __exit;
            }

            response = r;
        }

        memcpy(response + total_len, buffer, bytes);
        total_len += bytes;
    }

    response[total_len] = '\0';

__exit:
    if (sockfd >= 0) {
        close(sockfd);
    }
    return response;
}

// @TODO: Better Error handling
