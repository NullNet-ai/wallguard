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

boolean_t https_request(const char* hostname, int port, void* data, size_t len) {
    char response[4096];

    SSL_library_init();
    SSL_load_error_strings();
    OpenSSL_add_all_algorithms();

    SSL_CTX* ctx = SSL_CTX_new(TLS_client_method());
    if (ctx == NULL) {
        ERR_print_errors_fp(stderr);
        return WM_FALSE;
    }

    if (!SSL_CTX_set_min_proto_version(ctx, TLS1_2_VERSION)) {
        ERR_print_errors_fp(stderr);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    BIO* bio = BIO_new_ssl_connect(ctx);
    if (bio == NULL) {
        ERR_print_errors_fp(stderr);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    char conn_string[256];
    snprintf(conn_string, sizeof(conn_string), "%s:%d", hostname, port);
    BIO_set_conn_hostname(bio, conn_string);

    SSL* ssl;
    BIO_get_ssl(bio, &ssl);
    if (ssl == NULL) {
        ERR_print_errors_fp(stderr);
        BIO_free_all(bio);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    SSL_set_mode(ssl, SSL_MODE_AUTO_RETRY);

    if (BIO_do_connect(bio) <= 0) {
        ERR_print_errors_fp(stderr);
        BIO_free_all(bio);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    if (BIO_do_handshake(bio) <= 0) {
        ERR_print_errors_fp(stderr);
        BIO_free_all(bio);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    if (BIO_write(bio, data, len) <= 0) {
        if (!BIO_should_retry(bio)) {
            ERR_print_errors_fp(stderr);
            BIO_free_all(bio);
            SSL_CTX_free(ctx);
            return WM_FALSE;
        }
    }

    while (WM_TRUE) {
        int len = BIO_read(bio, response, sizeof(response) - 1);
        if (len <= 0) {
            break;
        }
        response[len] = '\0';
        printf("%s", response);
    }

    BIO_free_all(bio);
    SSL_CTX_free(ctx);
    EVP_cleanup();

    return WM_TRUE;
}

boolean_t http_request(const char* hostname, int port, void* data, size_t len) {
    int                sockfd, n;
    struct sockaddr_in serv_addr;
    struct hostent*    server;
    char               response[4096];

    // Create socket
    sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) {
        fprintf(stderr, "Cannot open a socket\n");
        return WM_FALSE;
    }

    // Get server by hostname
    server = gethostbyname(hostname);
    if (server == NULL) {
        fprintf(stderr, "ERROR, no such host\n");
        close(sockfd);

        return WM_FALSE;
    }

    bzero((char*)&serv_addr, sizeof(serv_addr));
    serv_addr.sin_family = AF_INET;
    bcopy((char*)server->h_addr, (char*)&serv_addr.sin_addr.s_addr, server->h_length);
    serv_addr.sin_port = htons(port);

    if (connect(sockfd, (struct sockaddr*)&serv_addr, sizeof(serv_addr)) < 0) {
        fprintf(stderr, "ERROR connecting");
        close(sockfd);
        return WM_FALSE;
    }

    n = write(sockfd, data, len);
    if (n < 0) {
        fprintf(stderr, "ERROR writing to socket");
        close(sockfd);
        return WM_FALSE;
    }

    bzero(response, sizeof(response));
    n = read(sockfd, response, sizeof(response) - 1);
    if (n < 0) {
        fprintf(stderr, "ERROR reading from socket");
        close(sockfd);
        return WM_FALSE;
    }

    printf("%s\n", response);
    close(sockfd);
    return WM_TRUE;
}

// @TODO: cleanup logig
// @TODO no bcopy and bzeroP
// @TODO: error logging / handling
