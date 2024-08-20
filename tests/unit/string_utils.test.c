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

void add_string_utils_tests() {
    CU_pSuite suite = CU_add_suite("Str Utils Tests", NULL, NULL);

    CU_add_test(suite, "test_generate_random_string_null_buffer", test_generate_random_string_null_buffer);
    CU_add_test(suite, "test_generate_random_string_length", test_generate_random_string_length);
    CU_add_test(suite, "test_generate_random_string_length_one", test_generate_random_string_length_one);
}