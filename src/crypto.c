#include "crypto.h"

#include <logger/logger.h>
#include <openssl/err.h>
#include <openssl/evp.h>
#include <openssl/rsa.h>
#include <openssl/core_names.h>
#include <openssl/param_build.h>
#include <openssl/pem.h>
#include <openssl/rand.h>
#include <openssl/hmac.h>
#include <openssl/decoder.h>

#define OSSL_LOG_ERROR(MSG_TITLE)                                      \
    do {                                                               \
        char err_msg[256] = {0};                                       \
        ERR_error_string_n(ERR_get_error(), err_msg, sizeof(err_msg)); \
        WALLMON_LOG_ERROR("%s Error: %s\n", MSG_TITLE, err_msg);       \
    } while (0)

char* base64_encode(const uint8_t* data, size_t len) {
    BIO* b64  = BIO_new(BIO_f_base64());
    BIO* bmem = BIO_new(BIO_s_mem());

    BIO_set_flags(b64, BIO_FLAGS_BASE64_NO_NL);
    b64 = BIO_push(b64, bmem);

    BIO_write(b64, data, len);
    BIO_flush(b64);

    BUF_MEM* bptr;
    BIO_get_mem_ptr(b64, &bptr);

    char* buffer = W_MALLOC(bptr->length + 1);
    memcpy(buffer, bptr->data, bptr->length);
    buffer[bptr->length] = '\0';

    BIO_free_all(b64);
    return buffer;
}

EVP_PKEY* decode_pem_key(const char* pem_key) {
    BIO* bio = BIO_new_mem_buf(pem_key, -1);
    if (!bio) {
        OSSL_LOG_ERROR("decode_pem_key: BIO_new_mem_buf failed");
        return NULL;
    }

    EVP_PKEY*         pkey = NULL;
    OSSL_DECODER_CTX* ctx  = OSSL_DECODER_CTX_new_for_pkey(&pkey, "PEM", NULL, "RSA", EVP_PKEY_PUBLIC_KEY, NULL, NULL);
    if (!ctx || !OSSL_DECODER_from_bio(ctx, bio)) {
        OSSL_LOG_ERROR("decode_pem_key: Failed to decode PEM failed");
        OSSL_DECODER_CTX_free(ctx);
        BIO_free(bio);
        return NULL;
    }

    OSSL_DECODER_CTX_free(ctx);
    BIO_free(bio);

    return pkey;
}

boolean_t pkey_encrypt(EVP_PKEY* pkey, const uint8_t* data, size_t size, uint8_t** output, size_t* output_size) {
    if (!pkey || !data || !size || !output || !output_size) {
        WALLMON_LOG_ERROR("pkey_encrypt: Invalid arguments");
        return WM_FALSE;
    }

    EVP_PKEY_CTX* ctx = EVP_PKEY_CTX_new(pkey, NULL);
    if (!ctx) {
        OSSL_LOG_ERROR("pkey_encrypt: Failed to create EVP_PKEY_CTX.");
        EVP_PKEY_free(pkey);
        return WM_FALSE;
    }

    if (EVP_PKEY_encrypt_init(ctx) <= 0) {
        OSSL_LOG_ERROR("pkey_encrypt: EVP_PKEY_encrypt_init failed.");
        EVP_PKEY_free(pkey);
        EVP_PKEY_CTX_free(ctx);
        return WM_FALSE;
    }

    if (EVP_PKEY_CTX_set_rsa_padding(ctx, RSA_PKCS1_OAEP_PADDING) <= 0) {
        OSSL_LOG_ERROR("pkey_encrypt: Failed to set OAEP padding.");
        EVP_PKEY_free(pkey);
        EVP_PKEY_CTX_free(ctx);
        return WM_FALSE;
    }

    *output_size = (size_t)(EVP_PKEY_get_bits(pkey) + 7) / 8;
    *output      = W_MALLOC(*output_size);

    if (EVP_PKEY_encrypt(ctx, *output, output_size, data, size) <= 0) {
        OSSL_LOG_ERROR("pkey_encrypt: Encryption failed.");
        free(*output);

        *output      = NULL;
        *output_size = 0;

        EVP_PKEY_CTX_free(ctx);
        return WM_FALSE;
    }

    EVP_PKEY_CTX_free(ctx);
    return WM_TRUE;
}

boolean_t encrypt_file(FILE* ifile, FILE* ofile, uint8_t key[W_AES_KEY_SIZE], uint8_t iv[W_AES_BLOCK_SIZE]) {
    if (RAND_bytes(iv, W_AES_BLOCK_SIZE) != 1) {
        OSSL_LOG_ERROR("encrypt_file: Failed to generate a random IV");
        return WM_FALSE;
    }

    if (RAND_bytes(key, W_AES_KEY_SIZE) != 1) {
        OSSL_LOG_ERROR("encrypt_file: Failed to generate a random AES key");
        return WM_FALSE;
    }

    EVP_CIPHER_CTX* ctx = EVP_CIPHER_CTX_new();
    if (!ctx) {
        OSSL_LOG_ERROR("encrypt_file: Failed to create EVP_CIPHER_CTX");
        return WM_FALSE;
    }

    if (EVP_EncryptInit_ex(ctx, EVP_aes_256_cbc(), NULL, key, iv) != 1) {
        OSSL_LOG_ERROR("encrypt_file: EVP_EncryptInit_ex failed");
        EVP_CIPHER_CTX_free(ctx);
        return WM_FALSE;
    }

    uint8_t input_buffer[1024];
    uint8_t output_buffer[1024 + W_AES_BLOCK_SIZE];
    int     bytes_read, output_len;

    while ((bytes_read = fread(input_buffer, 1, sizeof(input_buffer), ifile)) > 0) {
        if (EVP_EncryptUpdate(ctx, output_buffer, &output_len, input_buffer, bytes_read) != 1) {
            OSSL_LOG_ERROR("encrypt_file: EVP_EncryptUpdate failed");
            EVP_CIPHER_CTX_free(ctx);
            return WM_FALSE;
        }
        fwrite(output_buffer, 1, output_len, ofile);
    }

    if (EVP_EncryptFinal_ex(ctx, output_buffer, &output_len) != 1) {
        OSSL_LOG_ERROR("encrypt_file: EVP_EncryptFinal_ex failed");
        EVP_CIPHER_CTX_free(ctx);
        return WM_FALSE;
    }
    fwrite(output_buffer, 1, output_len, ofile);

    EVP_CIPHER_CTX_free(ctx);
    return WM_TRUE;
}
