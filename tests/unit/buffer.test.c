#include <CUnit/CUnit.h>
#include <CUnit/Basic.h>
#include <utils/dbuffer.h>
#include <stdlib.h>
#include <string.h>

#define TEST_CAPACITY 1024

static void test_buffer_init() {
    buffer_t buff;

    buffer_init(&buff, TEST_CAPACITY);

    CU_ASSERT_EQUAL(buff.capacity, TEST_CAPACITY);
    CU_ASSERT_EQUAL(buff.offset, 0);
    CU_ASSERT_PTR_NOT_NULL(buff.memory);

    buffer_free(&buff);
}

static void test_buffer_free() {
    buffer_t buff;

    buffer_init(&buff, TEST_CAPACITY);
    buffer_free(&buff);

    CU_ASSERT_EQUAL(buff.capacity, 0);
    CU_ASSERT_EQUAL(buff.offset, 0);
    CU_ASSERT_PTR_NULL(buff.memory);
}

static void test_buffer_can_write() {
    buffer_t buff;

    buffer_init(&buff, TEST_CAPACITY);

    CU_ASSERT(buffer_can_write(&buff, 512) == WM_TRUE);
    CU_ASSERT(buffer_can_write(&buff, TEST_CAPACITY) == WM_TRUE);
    CU_ASSERT(buffer_can_write(&buff, TEST_CAPACITY + 1) == WM_FALSE);

    buffer_free(&buff);
}

static void test_buffer_write() {
    buffer_t buff;
    char     data[128] = "Test Data";

    buffer_init(&buff, TEST_CAPACITY);

    CU_ASSERT(buffer_write(&buff, data, strlen(data) + 1) == WM_TRUE);
    CU_ASSERT_EQUAL(buff.offset, strlen(data) + 1);

    CU_ASSERT(buffer_write(&buff, data, TEST_CAPACITY) == WM_FALSE);
    CU_ASSERT_EQUAL(buff.offset, strlen(data) + 1);

    buffer_free(&buff);
}

void test_buffer_clear() {
    buffer_t buff;
    char     data[128] = "Test Data";

    buffer_init(&buff, TEST_CAPACITY);

    CU_ASSERT(buffer_write(&buff, data, strlen(data) + 1) == WM_TRUE);
    CU_ASSERT_EQUAL(buff.offset, strlen(data) + 1);

    buffer_clear(&buff);

    CU_ASSERT_EQUAL(buff.offset, 0);

    CU_ASSERT(buffer_write(&buff, data, strlen(data) + 1) == WM_TRUE);
    CU_ASSERT_EQUAL(buff.offset, strlen(data) + 1);

    buffer_free(&buff);
}

void add_buffer_tests(void) {
    CU_pSuite suite = CU_add_suite("Buffer Tests", 0, 0);
    CU_add_test(suite, "test_buffer_init", test_buffer_init);
    CU_add_test(suite, "test_buffer_free", test_buffer_free);
    CU_add_test(suite, "test_buffer_can_write", test_buffer_can_write);
    CU_add_test(suite, "test_buffer_write", test_buffer_write);
    CU_add_test(suite, "test_buffer_clear", test_buffer_clear);
}
