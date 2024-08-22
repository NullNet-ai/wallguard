#include "net_utils.test.h"

#include <CUnit/CUnit.h>
#include <CUnit/Basic.h>
#include <stdlib.h>
#include <string.h>

#include <utils/net.h>

static void test_parse_response_valid() {
    char   raw_response[] = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 13\r\n\r\nHello, world!";
    size_t response_len   = strlen(raw_response);

    http_response* response = parse_response(raw_response, response_len);

    CU_ASSERT_PTR_NOT_NULL(response);
    CU_ASSERT_EQUAL(response->status_code, 200);
    CU_ASSERT_EQUAL(response->headers_count, 2);
    CU_ASSERT_STRING_EQUAL(response->headers[0].key, "Content-Type");
    CU_ASSERT_STRING_EQUAL(response->headers[0].value, "text/html");
    CU_ASSERT_STRING_EQUAL(response->headers[1].key, "Content-Length");
    CU_ASSERT_STRING_EQUAL(response->headers[1].value, "13");
    CU_ASSERT_EQUAL(response->body_len, 13);
    CU_ASSERT_STRING_EQUAL(response->body, "Hello, world!");

    release_response(response);
}

static void test_parse_response_no_headers() {
    char   raw_response[] = "HTTP/1.1 200 OK\r\n\r\nHello, world!";
    size_t response_len   = strlen(raw_response);

    http_response* response = parse_response(raw_response, response_len);

    CU_ASSERT_PTR_NOT_NULL(response);
    CU_ASSERT_EQUAL(response->status_code, 200);
    CU_ASSERT_EQUAL(response->headers_count, 0);
    CU_ASSERT_EQUAL(response->body_len, 13);
    CU_ASSERT_STRING_EQUAL(response->body, "Hello, world!");

    release_response(response);
}

static void test_parse_response_no_body() {
    char   raw_response[] = "HTTP/1.1 204 No Content\r\n\r\n";
    size_t response_len   = strlen(raw_response);

    http_response* response = parse_response(raw_response, response_len);

    CU_ASSERT_PTR_NOT_NULL(response);
    CU_ASSERT_EQUAL(response->status_code, 204);
    CU_ASSERT_EQUAL(response->headers_count, 0);
    CU_ASSERT_EQUAL(response->body_len, 0);
    CU_ASSERT_PTR_NULL(response->body);

    release_response(response);
}

static void test_parse_response_invalid_format() {
    char   raw_response[] = "INVALID RESPONSE";
    size_t response_len   = strlen(raw_response);

    http_response* response = parse_response(raw_response, response_len);

    CU_ASSERT_PTR_NULL(response);
}

static void test_parse_response_missing_status_code() {
    char   raw_response[] = "HTTP/1.1 \r\n\r\n";
    size_t response_len   = strlen(raw_response);

    http_response* response = parse_response(raw_response, response_len);

    CU_ASSERT_PTR_NULL(response);
}

static void test_parse_response_missing_header_value() {
    char   raw_response[] = "HTTP/1.1 200 OK\r\nContent-Type:\r\n\r\n";
    size_t response_len   = strlen(raw_response);

    http_response* response = parse_response(raw_response, response_len);

    CU_ASSERT_PTR_NOT_NULL(response);
    CU_ASSERT_EQUAL(response->status_code, 200);
    CU_ASSERT_EQUAL(response->headers_count, 1);
    CU_ASSERT_STRING_EQUAL(response->headers[0].key, "Content-Type");
    CU_ASSERT_STRING_EQUAL(response->headers[0].value, "");
    CU_ASSERT_PTR_NULL(response->body);
    CU_ASSERT_EQUAL(response->body_len, 0);

    release_response(response);
}

static void test_parse_response_malformed_headers() {
    char   raw_response[] = "HTTP/1.1 200 OK\r\nContent-Type text/html\r\n\r\n";
    size_t response_len   = strlen(raw_response);

    http_response* response = parse_response(raw_response, response_len);

    CU_ASSERT_PTR_NOT_NULL(response);
    CU_ASSERT_EQUAL(response->status_code, 200);
    // Malformed headers are ignored
    CU_ASSERT_EQUAL(response->headers_count, 0);
    CU_ASSERT_PTR_NULL(response->body);
    CU_ASSERT_EQUAL(response->body_len, 0);

    release_response(response);
}

static void test_parse_response_large_body() {
    char   raw_response[] = "HTTP/1.1 200 OK\r\nContent-Length: 1000000\r\n\r\n";
    size_t response_len   = strlen(raw_response) + 1000000;

    char* full_response = malloc(response_len + 1);
    strcpy(full_response, raw_response);
    memset(full_response + strlen(raw_response), 'A', 1000000);
    full_response[response_len] = '\0';

    http_response* response = parse_response(full_response, response_len);

    CU_ASSERT_PTR_NOT_NULL(response);
    CU_ASSERT_EQUAL(response->status_code, 200);
    CU_ASSERT_EQUAL(response->headers_count, 1);
    CU_ASSERT_STRING_EQUAL(response->headers[0].key, "Content-Length");
    CU_ASSERT_STRING_EQUAL(response->headers[0].value, "1000000");
    CU_ASSERT_EQUAL(response->body_len, 1000000);
    CU_ASSERT_EQUAL(strncmp(response->body, full_response + strlen(raw_response), 1000000), 0);

    release_response(response);
    free(full_response);
}

void add_net_utils_tests() {
    CU_pSuite suite = CU_add_suite("Net Utils Tests", NULL, NULL);

    CU_add_test(suite, "test_parse_response_valid", test_parse_response_valid);
    CU_add_test(suite, "test_parse_response_no_headers", test_parse_response_no_headers);
    CU_add_test(suite, "test_parse_response_no_body", test_parse_response_no_body);
    CU_add_test(suite, "test_parse_response_invalid_format", test_parse_response_invalid_format);
    CU_add_test(suite, "test_parse_response_missing_status_code", test_parse_response_missing_status_code);
    CU_add_test(suite, "test_parse_response_missing_header_value", test_parse_response_missing_header_value);
    CU_add_test(suite, "test_parse_response_malformed_headers", test_parse_response_malformed_headers);
    CU_add_test(suite, "test_parse_response_large_body", test_parse_response_large_body);
}