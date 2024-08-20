#include <utils/str.h>

#include <CUnit/CUnit.h>
#include <CUnit/Basic.h>

#include "unit/string_utils.test.h"

int main() {
    CU_initialize_registry();

    add_string_utils_tests();

    CU_basic_run_tests();
    CU_cleanup_registry();

    return CU_get_error();
}