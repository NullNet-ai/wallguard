#include <network/tls.h>

#include <openssl/ssl.h>
#include <openssl/bio.h>
#include <openssl/err.h>

struct tls_handle {
    SSL_CTX* ctx;
    BIO*     bio;
};

boolean_t tls_start(tls_handle** handle, const char* hostname, int port) {
    if (!handle || !hostname || port < 0) {
        return WM_FALSE;
    }

    SSL_CTX* ctx = NULL;
    BIO*     bio = NULL;

    SSL_library_init();
    SSL_load_error_strings();
    OpenSSL_add_all_algorithms();

    ctx = SSL_CTX_new(TLS_client_method());
    if (ctx == NULL) {
        return WM_FALSE;
    }

    if (!SSL_CTX_set_min_proto_version(ctx, TLS1_2_VERSION)) {
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    bio = BIO_new_ssl_connect(ctx);
    if (bio == NULL) {
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    char conn_string[256];
    memset(conn_string, 0, sizeof(conn_string));
    snprintf(conn_string, sizeof(conn_string), "%s:%d", hostname, port);
    BIO_set_conn_hostname(bio, conn_string);

    SSL* ssl = NULL;
    BIO_get_ssl(bio, &ssl);
    if (ssl == NULL) {
        BIO_free_all(bio);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    SSL_set_mode(ssl, SSL_MODE_AUTO_RETRY);

    if (BIO_do_connect(bio) <= 0) {
        BIO_free_all(bio);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    if (BIO_do_handshake(bio) <= 0) {
        BIO_free_all(bio);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    *handle = malloc(sizeof(tls_handle));
    if (!*handle) {
        BIO_free_all(bio);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    (*handle)->bio = bio;
    (*handle)->ctx = ctx;

    return WM_TRUE;
}

ssize_t tls_write(tls_handle* handle, uint8_t* data, size_t len) {
    if (!handle || !data || len == 0) {
        return -1;
    }

    return BIO_write(handle->bio, data, len);
}

ssize_t tls_read(tls_handle* handle, uint8_t* data, size_t len) {
    if (!handle || !data || len == 0) {
        return -1;
    }

    return BIO_read(handle->bio, data, len);
}

boolean_t tls_end(tls_handle* handle) {
    if (!handle) {
        return WM_FALSE;
    }

    BIO_free_all(handle->bio);
    SSL_CTX_free(handle->ctx);

    free(handle);
    return WM_TRUE;
}
