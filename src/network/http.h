#ifndef NETWORK_HTTP_H
#define NETWORK_HTTP_H

#include <utils/common.h>

/**
 * @brief Enumeration of HTTP methods.
 */
typedef enum {
    HTTP_METHOD_GET,
    HTTP_METHOD_POST,
    HTTP_METHOD_PUT,
    HTTP_METHOD_DELETE,
} http_method;

/**
 * @brief Structure representing an HTTP header.
 */
typedef struct {
    const char* key;
    const char* value;
} http_header;

/**
 * @brief Structure representing an HTTP request.
 */
typedef struct {
    http_header* headers;
    size_t       headers_count;
    char*        body;
    size_t       body_len;
    http_method  method;
} http_request;

/**
 * @brief Structure representing an HTTP response.
 */
typedef struct {
    char*        body;
    size_t       body_len;
    http_header* headers;
    size_t       headers_count;
    int          status_code;
} http_response;

/**
 * @brief Makes an HTTP request to the specified hostname and path.
 *
 * @param hostname The hostname of the server to which the request is sent.
 * @param path The path on the server where the request is directed.
 * @param port The port number to connect to on the server.
 * @param tls A `boolean_t` indicating whether to use TLS.
 * @param request A pointer to an `http_request` structure containing the HTTP method, headers, and body.
 * @return A pointer to an `http_response` structure containing the server's response, or `NULL` on failure.
 *
 * @note The returned `http_response` structure must be freed using `release_response` to avoid memory leaks.
 */
http_response* fetch(const char* hostname, const char* path, int port, boolean_t tls, http_request* request);

#endif
