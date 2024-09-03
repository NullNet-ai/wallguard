#include "bootstrap.test.h"

#include <CUnit/Basic.h>
#include <CUnit/CUnit.h>

#include <platform/bootstrap.h>
#include <utils/file_utils.h>

#include <sys/stat.h>

#define LOCK_FOLDER "/var/lock/"
#define LOCK_FILENAME "wallmon.lock"

// Start with no lock
static int startup(void) {
    if (!directory_exists(LOCK_FOLDER) && (mkdir(LOCK_FOLDER, 0777) != 0)) {
        return -1;
    }

    if (file_exists(LOCK_FOLDER LOCK_FILENAME) && (remove(LOCK_FOLDER LOCK_FILENAME) != 0)) {
        return -1;
    }

    return 0;
}

static int teardown(void) {
    if (file_exists(LOCK_FOLDER LOCK_FILENAME) && (remove(LOCK_FOLDER LOCK_FILENAME) != 0)) {
        return -1;
    }

    return 0;
}

static void test_lock_api(void) {
    CU_ASSERT_FALSE(system_locked());

    platform_info info = {0};

    const char uuid[] = "f47ac10b-58cc-4372-a567-0e02b2c3d479";
    memcpy(info.uuid, uuid, sizeof(uuid));

    // Lock system and validate with current UUID
    CU_ASSERT_TRUE(lock_system(&info));
    CU_ASSERT_TRUE(system_locked());
    CU_ASSERT_TRUE(validate_lock(&info));

    // Now change the uuid and validate again

    const char uuid2[] = "e2a1f58c-9d3a-4c8f-8e1c-2b4a6f3b5918";
    memcpy(info.uuid, uuid2, sizeof(uuid2));

    CU_ASSERT_FALSE(validate_lock(&info));
}

void add_bootstrap_tests(void) {
    CU_pSuite suite = CU_add_suite("Bootstrap tests", startup, teardown);
    CU_add_test(suite, "test_lock_api", test_lock_api);
}
