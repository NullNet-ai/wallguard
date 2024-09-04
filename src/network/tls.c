#include <network/tls.h>

#include <openssl/ssl.h>
#include <openssl/bio.h>
#include <openssl/err.h>

#include <logger/logger.h>

static void log_ssl_error(const char* message) {
    unsigned long err_code;
    char          err_buf[256];

    if (message) {
        WLOG_ERROR("%s", message);
    }

    while ((err_code = ERR_get_error()) != 0) {
        ERR_error_string_n(err_code, err_buf, sizeof(err_buf));
        WLOG_ERROR("OpenSSL error: %s", err_buf);
    }
}

struct tls_handle {
    SSL_CTX* ctx;
    BIO*     bio;
};

boolean_t tls_start(tls_handle** handle, const char* hostname, int port) {
    if (!hostname || port < 0) {
        WLOG_ERROR("tls_start: invalid arguments.");
        return WM_FALSE;
    }

    SSL_CTX* ctx = NULL;
    BIO*     bio = NULL;

    SSL_library_init();
    SSL_load_error_strings();
    OpenSSL_add_all_algorithms();

    ctx = SSL_CTX_new(TLS_client_method());
    if (ctx == NULL) {
        log_ssl_error("Failed to create SSL context.");
        return WM_FALSE;
    }

    if (!SSL_CTX_set_min_proto_version(ctx, TLS1_2_VERSION)) {
        log_ssl_error("Failed to set minimum TLS version.");
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    bio = BIO_new_ssl_connect(ctx);
    if (bio == NULL) {
        log_ssl_error("Failed to create SSL BIO.");
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
        log_ssl_error("Failed to get SSL structure from BIO.");
        BIO_free_all(bio);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    SSL_set_mode(ssl, SSL_MODE_AUTO_RETRY);

    if (BIO_do_connect(bio) <= 0) {
        log_ssl_error("Failed to connect to the server.");
        BIO_free_all(bio);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    if (BIO_do_handshake(bio) <= 0) {
        log_ssl_error("TLS handshake failed.");
        BIO_free_all(bio);
        SSL_CTX_free(ctx);
        return WM_FALSE;
    }

    *handle = malloc(sizeof(tls_handle));
    if (!*handle) {
        WLOG_ERROR("Failed to allocate memory for TLS handle.");
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
        WLOG_ERROR("tls_write: invalid arguments.");
        return -1;
    }

    ssize_t result = BIO_write(handle->bio, data, len);

    if (result <= 0) {
        log_ssl_error("Failed to write data to TLS connection.");
    }

    return result;
}

ssize_t tls_read(tls_handle* handle, uint8_t* data, size_t len) {
    if (!handle || !data || len == 0) {
        WLOG_ERROR("tls_read: invalid arguments.");
        return -1;
    }

    ssize_t result = BIO_read(handle->bio, data, len);
    
    if (result <= 0) {
        log_ssl_error("Failed to read data from TLS connection.");
    }
    
    return result;
}

void tls_end(tls_handle* handle) {
    BIO_free_all(handle->bio);
    SSL_CTX_free(handle->ctx);
    free(handle);
}
