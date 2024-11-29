#include "net.test.h"

#include <utils/net.h>

#include <CUnit/CUnit.h>
#include <CUnit/Basic.h>

static void test_parse_url_valid_tcp(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("tcp://example.com:1234", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_TRUE);
    CU_ASSERT_STRING_EQUAL(hostname, "example.com");
    CU_ASSERT(port == 1234);
    CU_ASSERT(tls == WM_FALSE); // Assuming TCP is not considered TLS in your implementation
}

static void test_parse_url_valid_udp(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("udp://example.org:5678", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_TRUE);
    CU_ASSERT_STRING_EQUAL(hostname, "example.org");
    CU_ASSERT(port == 5678);
    CU_ASSERT(tls == WM_FALSE); // Assuming UDP is not TLS
}

static void test_parse_url_valid_http(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("http://example.com", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_TRUE);
    CU_ASSERT_STRING_EQUAL(hostname, "example.com");
    CU_ASSERT(port == 80);
    CU_ASSERT(tls == WM_FALSE);
}

static void test_parse_url_valid_https(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("https://example.com", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_TRUE);
    CU_ASSERT_STRING_EQUAL(hostname, "example.com");
    CU_ASSERT(port == 443);
    CU_ASSERT(tls == WM_TRUE);
}

static void test_parse_url_with_port(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("https://example.com:8443", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_TRUE);
    CU_ASSERT_STRING_EQUAL(hostname, "example.com");
    CU_ASSERT(port == 8443);
    CU_ASSERT(tls == WM_TRUE);
}

static void test_parse_url_with_path(void) {
    char      hostname[256];
    char      path[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("https://example.com:8443/path/to/resource", hostname, sizeof(hostname), path, sizeof(path),
                       &port, &tls);
    CU_ASSERT(result == WM_TRUE);
    CU_ASSERT_STRING_EQUAL(hostname, "example.com");
    CU_ASSERT_STRING_EQUAL(path, "/path/to/resource");
    CU_ASSERT(port == 8443);
    CU_ASSERT(tls == WM_TRUE);
}

static void test_parse_url_invalid_protocol(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("ftp://example.com", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_FALSE);
}

static void test_parse_url_invalid_url_format(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("http:/example.com", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_FALSE);
}

static void test_parse_url_empty_string(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_FALSE);
}

static void test_parse_url_no_hostname(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("http://:8080", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_FALSE);
}

static void test_parse_url_no_protocol(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("example.com", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_FALSE);
}

static void test_parse_url_long_hostname(void) {
    char      hostname[256];
    int       port;
    boolean_t tls;
    boolean_t result;

    char long_hostname[300];
    memset(long_hostname, 'a', 299);
    long_hostname[299] = '\0';

    result = parse_url(long_hostname, hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_FALSE);
}

static void test_parse_url_small_hostname_buffer(void) {
    char      hostname[5];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("https://example.com", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_FALSE);
}

static void test_parse_url_small_path_buffer(void) {
    char      hostname[256];
    char      path[10];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("https://example.com/this/is/a/very/long/path", hostname, sizeof(hostname), path, sizeof(path),
                       &port, &tls);
    CU_ASSERT(result == WM_FALSE);
}

static void test_parse_url_exact_hostname_buffer(void) {
    char      hostname[12];
    int       port;
    boolean_t tls;
    boolean_t result;

    result = parse_url("https://example.com", hostname, sizeof(hostname), NULL, 0, &port, &tls);
    CU_ASSERT(result == WM_TRUE);
    CU_ASSERT_STRING_EQUAL(hostname, "example.com");
}

static void test_parse_url_exact_path_buffer(void) {
    char      hostname[256];
    char      path[21];
    int       port;
    boolean_t tls;
    boolean_t result;

    result =
        parse_url("https://example.com/this/is/a/path", hostname, sizeof(hostname), path, sizeof(path), &port, &tls);
    CU_ASSERT(result == WM_TRUE);
    CU_ASSERT_STRING_EQUAL(path, "/this/is/a/path");
}

static void test_parse_url_missing_hostname(void) {
    char      url[] = "https://";
    char      hostname[50];
    char      path[50];
    int       port;
    boolean_t tls;

    boolean_t result = parse_url(url, hostname, sizeof(hostname), path, sizeof(path), &port, &tls);

    CU_ASSERT_EQUAL(result, WM_FALSE);
}

static void test_parse_url_ignore_port(void) {
    char      url[] = "https://example:8000/";
    char      hostname[50];
    char      path[50];
    boolean_t tls;

    boolean_t result = parse_url(url, hostname, sizeof(hostname), path, sizeof(path), NULL, &tls);

    CU_ASSERT_EQUAL(result, WM_TRUE);
}

static void test_parse_url_ignore_tls(void) {
    char url[] = "https://example:8000/";
    char hostname[50];
    char path[50];
    int  port;

    boolean_t result = parse_url(url, hostname, sizeof(hostname), path, sizeof(path), &port, NULL);

    CU_ASSERT_EQUAL(result, WM_TRUE);
}

void add_net_utils_tests(void) {
    CU_pSuite suite = CU_add_suite("NET Utils Tests", NULL, NULL);

    CU_add_test(suite, "test_parse_url_valid_udp", test_parse_url_valid_udp);
    CU_add_test(suite, "test_parse_url_valid_tcp", test_parse_url_valid_tcp);
    CU_add_test(suite, "test_parse_url_valid_http", test_parse_url_valid_http);
    CU_add_test(suite, "test_parse_url_valid_https", test_parse_url_valid_https);
    CU_add_test(suite, "test_parse_url_with_port", test_parse_url_with_port);
    CU_add_test(suite, "test_parse_url_with_path", test_parse_url_with_path);
    CU_add_test(suite, "test_parse_url_invalid_protocol", test_parse_url_invalid_protocol);
    CU_add_test(suite, "test_parse_url_invalid_url_format", test_parse_url_invalid_url_format);
    CU_add_test(suite, "test_parse_url_empty_string", test_parse_url_empty_string);
    CU_add_test(suite, "test_parse_url_no_hostname", test_parse_url_no_hostname);
    CU_add_test(suite, "test_parse_url_no_protocol", test_parse_url_no_protocol);
    CU_add_test(suite, "test_parse_url_long_hostname", test_parse_url_long_hostname);
    CU_add_test(suite, "test_parse_url_small_hostname_buffer", test_parse_url_small_hostname_buffer);
    CU_add_test(suite, "test_parse_url_small_path_buffer", test_parse_url_small_path_buffer);
    CU_add_test(suite, "test_parse_url_exact_hostname_buffer", test_parse_url_exact_hostname_buffer);
    CU_add_test(suite, "test_parse_url_exact_path_buffer", test_parse_url_exact_path_buffer);
    CU_add_test(suite, "test_parse_url_missing_hostname", test_parse_url_missing_hostname);
    CU_add_test(suite, "test_parse_url_ignore_port", test_parse_url_ignore_port);
    CU_add_test(suite, "test_parse_url_ignore_tls", test_parse_url_ignore_tls);
}
