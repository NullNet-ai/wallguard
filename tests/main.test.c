#include <utils/str.h>

#include <CUnit/CUnit.h>
#include <CUnit/Basic.h>

#include "unit/string_utils.test.h"
#include "unit/url_utils.test.h"
#include "unit/net_utils.test.h"
#include "unit/file_utils.test.h"

#include "integration/fetch.test.h"

int main() {
    CU_initialize_registry();

    // Unit
    add_string_utils_tests();
    add_url_utils_tests();
    add_net_utils_tests();
    add_file_utils_tests();

    // Integration
    add_fetch_tests();

    CU_basic_set_mode(CU_BRM_VERBOSE);
    CU_basic_run_tests();
    CU_cleanup_registry();

    return CU_get_error();
}
