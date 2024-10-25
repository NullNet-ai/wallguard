#include "file_utils.test.h"

#include <CUnit/Basic.h>

#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <sys/types.h>

#include <utils/file_utils.h>

static void test_file_exists(void) {
    const char *existing_file     = "/tmp/existing_file.txt";
    const char *non_existing_file = "/tmp/non_existing_file.txt";

    FILE *file = fopen(existing_file, "w");
    fclose(file);

    CU_ASSERT_TRUE(file_exists(existing_file));
    CU_ASSERT_FALSE(file_exists(non_existing_file));

    remove(existing_file);
}

static void test_directory_exists(void) {
    const char *existing_dir     = "/tmp/test_dir";
    const char *non_existing_dir = "/tmp/non_existing_dir";

    mkdir(existing_dir, 0777);

    CU_ASSERT_TRUE(directory_exists(existing_dir));
    CU_ASSERT_FALSE(directory_exists(non_existing_dir));

    rmdir(existing_dir);
}

static void test_file_size(void) {
    const char *file_path = "/tmp/test_file.txt";

    FILE *file = fopen(file_path, "w");
    fputs("Hello, World!", file);
    fclose(file);

    CU_ASSERT_EQUAL(file_size(file_path), 13);

    CU_ASSERT_EQUAL(file_size("/tmp/non_existing_file.txt"), -1);

    remove(file_path);
}

static void test_filename(void) {
    CU_ASSERT_STRING_EQUAL(filename("/tmp/test_file.txt"), "test_file.txt");
    CU_ASSERT_STRING_EQUAL(filename("test_file.txt"), "test_file.txt");
    CU_ASSERT_STRING_EQUAL(filename("/another/path/to/test_file.txt"), "test_file.txt");
}

static void test_copy_file(void) {
    const char *source_file = "/tmp/source_file.txt";
    const char *dest_file   = "/tmp/dest_file.txt";

    FILE *file = fopen(source_file, "w");
    fputs("Sample content", file);
    fclose(file);

    CU_ASSERT_TRUE(copy_file(source_file, dest_file));

    CU_ASSERT_TRUE(file_exists(dest_file));
    CU_ASSERT_EQUAL(file_size(dest_file), 14);

    remove(source_file);
    remove(dest_file);
}

static void test_file_monitor_init(void) {
    const char *file_path = "/tmp/monitor_file.txt";

    FILE *file = fopen(file_path, "w");
    fputs("Monitor this file", file);
    fclose(file);

    file_monitor monitor;
    CU_ASSERT_TRUE(file_monitor_init(&monitor, file_path));
    CU_ASSERT_PTR_NOT_NULL(monitor.filepath);
    CU_ASSERT_EQUAL(file_size(monitor.filepath), 17);

    CU_ASSERT_FALSE(file_monitor_init(NULL, file_path));
    CU_ASSERT_FALSE(file_monitor_init(&monitor, NULL));

    remove(file_path);
}

static void test_file_monitor_check(void) {
    const char *file_path = "/tmp/monitor_check_file.txt";

    FILE *file = fopen(file_path, "w");
    fputs("Initial content", file);
    fclose(file);

    file_monitor monitor;
    file_monitor_init(&monitor, file_path);

    CU_ASSERT_EQUAL(file_monitor_check(&monitor), 0);

    file = fopen(file_path, "w");
    fputs("Modified content", file);
    fclose(file);

    // Since `stat.st_mtime` is in seconds, it's unlikely that a full second has passed since the last modification.
    // To simulate a file modification without delaying execution, we subtract one second from `monitor.last_update`.
    // This avoids the need to use the `sleep` function, which would slow down the test execution.
    monitor.last_update -= 1;

    CU_ASSERT_EQUAL(file_monitor_check(&monitor), 1);
    CU_ASSERT_EQUAL(file_monitor_check(NULL), -1);

    remove(file_path);
}

static void test_read_file_content(void) {
    const char *file_path = "/tmp/content_file.txt";

    FILE *file = fopen(file_path, "w");
    fputs("This is the file content", file);
    fclose(file);

    uint8_t *content = read_file_content(file_path);
    CU_ASSERT_PTR_NOT_NULL(content);
    CU_ASSERT_STRING_EQUAL((char *)content, "This is the file content");

    free(content);

    CU_ASSERT_PTR_NULL(read_file_content("/tmp/non_existing_file.txt"));

    remove(file_path);
}

static void test_extension(void) {
    const char *ext;

    ext = extension("file.txt");
    CU_ASSERT_STRING_EQUAL(ext, "txt");

    ext = extension("hello.world.cpp");
    CU_ASSERT_STRING_EQUAL(ext, "cpp");

    ext = extension("file");
    CU_ASSERT_PTR_NULL(ext);

    ext = extension("");
    CU_ASSERT_PTR_NULL(ext);

    ext = extension(".");
    CU_ASSERT_PTR_NULL(ext);

    ext = extension("file.");
    CU_ASSERT_PTR_NULL(ext);
}

void add_file_utils_tests(void) {
    CU_pSuite suite = CU_add_suite("File Utils Tests", NULL, NULL);

    CU_add_test(suite, "test_file_exists", test_file_exists);
    CU_add_test(suite, "test_directory_exists", test_directory_exists);
    CU_add_test(suite, "test_file_size", test_file_size);
    CU_add_test(suite, "test_filename", test_filename);
    CU_add_test(suite, "test_copy_file", test_copy_file);
    CU_add_test(suite, "test_file_monitor_init", test_file_monitor_init);
    CU_add_test(suite, "test_file_monitor_check", test_file_monitor_check);
    CU_add_test(suite, "test_read_file_content", test_read_file_content);
    CU_add_test(suite, "test_extension", test_extension);
}