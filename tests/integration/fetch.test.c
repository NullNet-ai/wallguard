#include "fetch.test.h"
#include <network/http.h>
#include <utils/net.h>
#include <CUnit/Basic.h>

static void test_fetch_https_request() {
    // https://example.com

    http_header headers[] = {
        {.key = "Connection", .value = "close"},
        {.key = "Host", .value = "example.com"},
    };

    http_request request;
    request.method        = HTTP_METHOD_GET;
    request.body          = NULL;
    request.body_len      = 0;
    request.headers       = headers;
    request.headers_count = ARRAY_SIZE(headers);

    http_response* response = fetch("example.com", "/", 443, WM_TRUE, &request);
    CU_ASSERT_PTR_NOT_NULL(response);

    if (response) {
        release_response(response);
    }
}

static void test_fetch_non_existing_domain_https() {
    http_header headers[] = {
        {.key = "Connection", .value = "close"},
        {.key = "Host", .value = "hello.example"},
    };

    http_request request;
    request.method        = HTTP_METHOD_GET;
    request.body          = NULL;
    request.body_len      = 0;
    request.headers       = headers;
    request.headers_count = ARRAY_SIZE(headers);

    http_response* response = fetch("hello.example", "/", 443, WM_TRUE, &request);
    CU_ASSERT_PTR_NULL(response);
}

static void test_fetch_http_request() {
    // http://httpbin.org

    http_header headers[] = {
        {.key = "Connection", .value = "close"},
        {.key = "Host", .value = "httpbin.org"},
    };

    http_request request;
    request.method        = HTTP_METHOD_GET;
    request.body          = NULL;
    request.body_len      = 0;
    request.headers       = headers;
    request.headers_count = ARRAY_SIZE(headers);

    http_response* response = fetch("httpbin.org", "/", 80, WM_FALSE, &request);
    CU_ASSERT_PTR_NOT_NULL(response);

    if (response) {
        release_response(response);
    }
}

static void test_fetch_non_existing_domain_http() {
    http_header headers[] = {
        {.key = "Connection", .value = "close"},
        {.key = "Host", .value = "hello.example"},
    };

    http_request request;
    request.method        = HTTP_METHOD_GET;
    request.body          = NULL;
    request.body_len      = 0;
    request.headers       = headers;
    request.headers_count = ARRAY_SIZE(headers);

    http_response* response = fetch("hello.example", "/", 80, WM_FALSE, &request);
    CU_ASSERT_PTR_NULL(response);
}

void add_fetch_tests(void) {
    CU_pSuite suite = CU_add_suite("Fetch tests", NULL, NULL);
    CU_add_test(suite, "test_fetch_https_request", test_fetch_https_request);
    CU_add_test(suite, "test_fetch_non_existing_domain_https", test_fetch_non_existing_domain_https);

    CU_add_test(suite, "test_fetch_http_request", test_fetch_http_request);
    CU_add_test(suite, "test_fetch_non_existing_domain_http", test_fetch_non_existing_domain_http);
}
