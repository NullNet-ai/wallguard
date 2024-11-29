#include "string_utils.test.h"

#include <CUnit/CUnit.h>
#include <CUnit/Basic.h>

#include <utils/str.h>

static void test_string_to_number_valid(void) {
    long result = 0;

    CU_ASSERT_TRUE(string_to_integer("1234", &result, 10));
    CU_ASSERT_EQUAL(result, 1234);

    CU_ASSERT_TRUE(string_to_integer("-99", &result, 10));
    CU_ASSERT_EQUAL(result, -99);
}

static void test_string_to_number_invalid(void) {
    long result = 0;

    CU_ASSERT_FALSE(string_to_integer("abc", &result, 10));
    CU_ASSERT_FALSE(string_to_integer("123abc", &result, 10));
    CU_ASSERT_FALSE(string_to_integer("", &result, 10));
}

void test_valid_uuid() {
    const char* valid_uuid = "123e4567-e89b-12d3-a456-426614174000";
    CU_ASSERT_TRUE(is_valid_uuid(valid_uuid));
}

void test_invalid_uuid_length() {
    const char* invalid_short_uuid = "123e4567-e89b-12d3-a456-42661417400";    // 1 char short
    const char* invalid_long_uuid  = "123e4567-e89b-12d3-a456-4266141740000";  // 1 char long
    CU_ASSERT_FALSE(is_valid_uuid(invalid_short_uuid));
    CU_ASSERT_FALSE(is_valid_uuid(invalid_long_uuid));
}

void test_invalid_uuid_dash_position() {
    const char* invalid_uuid_dash = "123e4567e89b-12d3-a456-426614174000";
    CU_ASSERT_FALSE(is_valid_uuid(invalid_uuid_dash));
}

void test_invalid_uuid_non_hex() {
    const char* invalid_uuid_non_hex = "123e4567-e89b-12d3-a456-4266141740zz";
    CU_ASSERT_FALSE(is_valid_uuid(invalid_uuid_non_hex));
}

void test_null_uuid() { CU_ASSERT_FALSE(is_valid_uuid(NULL)); }

void test_valid_uppercase_uuid() {
    const char* valid_uppercase_uuid = "123E4567-E89B-12D3-A456-426614174000";
    CU_ASSERT_TRUE(is_valid_uuid(valid_uppercase_uuid));
}

static void test_uuid_to_bytes_valid(void) {
    const char* uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    uint8_t uuid_bytes[16];
    uint8_t expected_bytes[16] = {
        0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4,
        0xa7, 0x16, 0x44, 0x66, 0x55, 0x44, 0x00, 0x00};

    uuid_to_bytes(uuid_str, uuid_bytes);
    CU_ASSERT_NSTRING_EQUAL(uuid_bytes, expected_bytes, 16);
}

static void test_string_copy_valid() {
    const char* original = "Hello, world!";
    char*       copy     = string_copy(original);

    CU_ASSERT_PTR_NOT_NULL(copy);
    CU_ASSERT_STRING_EQUAL(copy, original);

    W_FREE(copy);
}

void add_string_utils_tests() {
    CU_pSuite suite = CU_add_suite("Str Utils Tests", NULL, NULL);

    CU_add_test(suite, "test_string_to_number_valid", test_string_to_number_valid);
    CU_add_test(suite, "test_string_to_number_invalid", test_string_to_number_invalid);

    CU_add_test(suite, "test_valid_uuid", test_valid_uuid);
    CU_add_test(suite, "test_invalid_uuid_length", test_invalid_uuid_length);
    CU_add_test(suite, "test_invalid_uuid_dash_position", test_invalid_uuid_dash_position);
    CU_add_test(suite, "test_invalid_uuid_non_hex", test_invalid_uuid_non_hex);
    CU_add_test(suite, "test_null_uuid", test_null_uuid);
    CU_add_test(suite, "test_valid_uppercase_uuid", test_valid_uppercase_uuid);
    CU_add_test(suite, "test_uuid_to_bytes_valid", test_uuid_to_bytes_valid);
    CU_add_test(suite, "test_string_copy_valid", test_string_copy_valid);
}
