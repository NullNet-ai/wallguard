#include "upload_sequence.h"
#include "server_requests.h"
#include "crypto.h"

#include <logger/logger.h>
#include <openssl/evp.h>

static boolean_t encrypt_config(const char* spath, const char* dpath, uint8_t key[W_AES_KEY_SIZE],
                                uint8_t iv[W_AES_BLOCK_SIZE]) {
    FILE* source_file = fopen(spath, "rb");
    if (!source_file) {
        WALLMON_LOG_ERROR("encrypt_config: Failed to open source file %s", spath);
        return WM_FALSE;
    }

    FILE* dest_file = fopen(dpath, "wb");
    if (!dest_file) {
        WALLMON_LOG_ERROR("encrypt_config: Failed to destination source file %s", dpath);
        fclose(source_file);
        return WM_FALSE;
    }

    boolean_t retval = encrypt_file(source_file, dest_file, key, iv);

    fclose(source_file);
    fclose(dest_file);

    if (!retval) {
        remove(dpath);
    }

    return retval;
}

boolean_t upload_sequence(const char* session_token, const char* path, const platform_info* info) {
    char temp_filename[L_tmpnam];
    if (tmpnam(temp_filename) == NULL) {
        WALLMON_LOG_ERROR("upload_sequence: Failed to generate temporary filename");
        return WM_FALSE;
    }

    char* public_key = NULL;
    if (!wallmon_fetch_key(session_token, &public_key)) {
        WALLMON_LOG_ERROR("upload_sequence: Failed to fetch public key");
        return WM_FALSE;
    }

    EVP_PKEY* pkey = decode_pem_key(public_key);
    if (!pkey) {
        WALLMON_LOG_ERROR("upload_sequence: Failed to decode public key");
        free(public_key);
        return WM_FALSE;
    }

    uint8_t sym_key[W_AES_KEY_SIZE];
    uint8_t sym_iv[W_AES_BLOCK_SIZE];

    if (!encrypt_config(path, temp_filename, sym_key, sym_iv)) {
        WALLMON_LOG_ERROR("upload_sequence: Failed to encrypt file %s", path);
        EVP_PKEY_free(pkey);
        free(public_key);
        return WM_FALSE;
    }

    uint8_t *sym_key_encoded, *sym_iv_encoded;
    size_t   sym_key_encoded_size, sym_iv_encoded_size;

    if (!pkey_encrypt(pkey, sym_key, sizeof(sym_key), &sym_key_encoded, &sym_key_encoded_size)) {
        WALLMON_LOG_ERROR("upload_sequence: Failed to encrypt AES key");
        EVP_PKEY_free(pkey);
        free(public_key);
        remove(temp_filename);
        return WM_FALSE;
    }

    if (!pkey_encrypt(pkey, sym_iv, sizeof(sym_iv), &sym_iv_encoded, &sym_iv_encoded_size)) {
        WALLMON_LOG_ERROR("upload_sequence: Failed to encrypt IV");
        EVP_PKEY_free(pkey);
        free(public_key);
        free(sym_key_encoded);
        remove(temp_filename);
        return WM_FALSE;
    }

    char* b64_key = base64_encode(sym_key_encoded, sym_key_encoded_size);
    char* b64_iv  = base64_encode(sym_iv_encoded, sym_iv_encoded_size);

    boolean_t retval = wallmon_upload_configuration(session_token, temp_filename, b64_key, b64_iv, info);
    if (!retval) {
        WALLMON_LOG_ERROR("upload_sequence: Failed to uplaod configuration to the server");
    }

    EVP_PKEY_free(pkey);
    free(public_key);
    free(sym_key_encoded);
    free(sym_iv_encoded);
    free(b64_key);
    free(b64_iv);
    remove(temp_filename);

    return retval;
}