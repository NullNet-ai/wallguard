#include <stdlib.h>

#include <CUnit/CUnit.h>
#include <CUnit/Basic.h>

#include "unit/string_utils.test.h"
#include "unit/file_utils.test.h"
#include "unit/crypto.test.h"

int main() {
    CU_initialize_registry();

    // Unit
    add_string_utils_tests();
    add_file_utils_tests();
    add_crypto_tests();

    CU_basic_set_mode(CU_BRM_VERBOSE);
    CU_basic_run_tests();

    CU_ErrorCode error    = CU_get_error();
    int          failures = CU_get_number_of_failures();

    CU_cleanup_registry();

    if (error != CUE_SUCCESS || failures > 0) {
        return EXIT_FAILURE;
    } else {
        return EXIT_SUCCESS;
    }
}
