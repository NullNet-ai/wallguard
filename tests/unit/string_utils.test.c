#include "string_utils.test.h"

#include <CUnit/CUnit.h>
#include <CUnit/Basic.h>

#include <utils/str.h>

static void test_generate_random_string_null_buffer(void) {
    generate_random_string(NULL, 10);
    CU_PASS("Buffer is NULL, function should return without crashing.");
}

static void test_generate_random_string_length(void) {
    char buf[11];
    generate_random_string(buf, 11);

    CU_ASSERT_EQUAL(strlen(buf), 10);
    CU_ASSERT(buf[10] == '\0');
}

static void test_generate_random_string_length_one(void) {
    char buf[1];
    generate_random_string(buf, 1);

    CU_ASSERT_EQUAL(strlen(buf), 0);
    CU_ASSERT(buf[0] == '\0');
}

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

void add_string_utils_tests() {
    CU_pSuite suite = CU_add_suite("Str Utils Tests", NULL, NULL);

    CU_add_test(suite, "test_generate_random_string_null_buffer", test_generate_random_string_null_buffer);
    CU_add_test(suite, "test_generate_random_string_length", test_generate_random_string_length);
    CU_add_test(suite, "test_generate_random_string_length_one", test_generate_random_string_length_one);

    CU_add_test(suite, "test_string_to_number_valid", test_string_to_number_valid);
    CU_add_test(suite, "test_string_to_number_invalid", test_string_to_number_invalid);
}
