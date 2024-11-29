#ifndef _CRYPTO_H_
#define _CRYPTO_H_

#include <utils/common.h>
#include <openssl/rsa.h>

#define W_AES_BLOCK_SIZE 16
#define W_AES_KEY_SIZE 32

/**
 * @brief Encodes binary data to a Base64-encoded string.
 *
 * @param data A pointer to the binary data to be encoded.
 * @param len The length of the binary data in bytes.
 * @return A pointer to a dynamically allocated, null-terminated string
 *         containing the Base64-encoded data. The caller is responsible for
 *         freeing the memory.
 */
char* base64_encode(const uint8_t* data, size_t len);

/**
 * @brief Decodes a PEM-formatted RSA public key into an `EVP_PKEY` structure.
 *
 *
 * @param pem_key: A null-terminated C string containing the PEM-encoded RSA public key.
 *
 * @return `EVP_PKEY*` A pointer to the decoded `EVP_PKEY` structure on success, or `NULL` on failure.
 *                     Caller is responsible for freeing the returned `EVP_PKEY` using `EVP_PKEY_free`.
 */
EVP_PKEY* decode_pem_key(const char* pem_key);

/**
 * @brief Encrypts data using the provided `EVP_PKEY` RSA public key.
 *
 * @param pkey         An initialized `EVP_PKEY` structure containing an RSA public key.
 *                     Must be a valid RSA public key; caller must manage its lifecycle.
 * @param data         Pointer to the input data buffer to be encrypted.
 * @param size         Size of the input data buffer in bytes.
 * @param output       Pointer to a location where the function will store the pointer to the
 *                     allocated output buffer containing the encrypted data. The caller is
 *                     responsible for freeing this buffer using free().
 * @param output_size  Pointer to a variable where the function will store the size of the
 *                     encrypted output in bytes.
 *
 * @return `WM_TRUE` on success, or `WM_FALSE` on failure. On failure.
 */
boolean_t pkey_encrypt(EVP_PKEY* pkey, const uint8_t* data, size_t size, uint8_t** output, size_t* output_size);

/**
 * @brief Encrypts data from an input file and writes the encrypted data to an output file using AES-256-CBC.
 *
 * @param ifile A pointer to the input file stream to be encrypted. Must be opened in binary read mode.
 * @param ofile A pointer to the output file stream where encrypted data will be written. Must be opened in binary write
 * mode.
 * @param key An output buffer where the generated AES key will be stored. Must be `W_AES_KEY_SIZE` bytes (32 bytes for
 * AES-256).
 * @param iv An output buffer where the generated AES IV (initialization vector) will be stored.
 *           The buffer must be `W_AES_BLOCK_SIZE` bytes (16 bytes for AES-CBC).
 * @return `WM_TRUE` if encryption succeeds, `WM_FALSE` if an error occurs during encryption or file I/O.
 */
boolean_t encrypt_file(FILE* ifile, FILE* ofile, uint8_t key[W_AES_KEY_SIZE], uint8_t iv[W_AES_BLOCK_SIZE]);

#endif
