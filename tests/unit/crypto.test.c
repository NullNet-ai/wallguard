#include "crypto.test.h"
#include "crypto.h"

#include <utils/common.h>
#include <CUnit/CUnit.h>
#include <CUnit/Basic.h>
#include <openssl/evp.h>
#include <openssl/err.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char* valid_pem_key =
    "-----BEGIN PUBLIC KEY-----\n"
    "MFwwDQYJKoZIhvcNAQEBBQADSwAwSAJBAMV35kV5xocpUlChxmzo8DFNjKaftYFbvQ/ohuM5c4uwDj8d7vnPnB6lNn1kGDRwPWEVqkgZ/i8Gi/"
    "v5t6gS1OcCAwEAAQ==\n"
    "-----END PUBLIC KEY-----\n";

static void test_base64_encode_known_input(void) {
    const uint8_t input[]         = "hello";
    const char*   expected_output = "aGVsbG8=";

    char* encoded = base64_encode(input, strlen((const char*)input));

    CU_ASSERT_PTR_NOT_NULL(encoded);
    CU_ASSERT_NSTRING_EQUAL(encoded, expected_output, 8);

    free(encoded);
}

static void test_base64_encode_empty_input(void) {
    const uint8_t input[]         = "";
    const char*   expected_output = "";

    char* encoded = base64_encode(input, 0);
    CU_ASSERT_PTR_NOT_NULL(encoded);
    CU_ASSERT_STRING_EQUAL(encoded, expected_output);

    free(encoded);
}

static void test_decode_pem_key_valid_input(void) {
    EVP_PKEY* pkey = decode_pem_key(valid_pem_key);
    CU_ASSERT_PTR_NOT_NULL(pkey);
    CU_ASSERT(EVP_PKEY_base_id(pkey) == EVP_PKEY_RSA);

    EVP_PKEY_free(pkey);
}

static void test_decode_pem_key_invalid_input(void) {
    const char* invalid_pem_key = "Invalid PEM format data";
    EVP_PKEY*   pkey            = decode_pem_key(invalid_pem_key);

    CU_ASSERT_PTR_NULL(pkey);
}

static void test_decode_pem_key_null_input(void) {
    EVP_PKEY* pkey = decode_pem_key(NULL);
    CU_ASSERT_PTR_NULL(pkey);
}

void add_crypto_tests(void) {
    CU_pSuite suite = CU_add_suite("File Utils Tests", NULL, NULL);

    CU_add_test(suite, "test_base64_encode_known_input", test_base64_encode_known_input);
    CU_add_test(suite, "test_base64_encode_empty_input", test_base64_encode_empty_input);

    CU_add_test(suite, "test_decode_pem_key_valid_input", test_decode_pem_key_valid_input);
    CU_add_test(suite, "test_decode_pem_key_invalid_input", test_decode_pem_key_invalid_input);
    CU_add_test(suite, "test_decode_pem_key_null_input", test_decode_pem_key_null_input);
}