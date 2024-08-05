#ifndef NETWORK_HTTP_H
#define NETWORK_HTTP_H

#include <utils/common.h>

/**
 * @brief Enumeration for HTTP methods.
 */
typedef enum {
    HTTP_METHOD_GET,
    HTTP_METHOD_POST,
    HTTP_METHOD_PUT,
    HTTP_METHOD_DELETE,
} http_method;

/**
 * @brief Structure to represent an HTTP header.
 */
typedef struct {
    const char* key;
    const char* value;
} http_header;

/**
 * @brief Structure to represent an HTTP request.
 */
typedef struct {
    http_header* headers;
    size_t       hlen;
    uint8_t*     body;
    size_t       blen;
    http_method  method;
    const char*  url;
} http_request;

/**
 * @brief Structure to represent an HTTP response.
 */
typedef struct {
    uint8_t*     __ptr;
    http_header* headers;
    size_t       hlen;
    uint8_t*     body;
    size_t       blen;
    int          status;
} http_response;

/**
 * @brief Fetches an HTTP response based on the given request.
 *
 * This function sends an HTTP request based on the provided `http_request` structure
 * and returns the response in an http_response structure. The caller is responsible
 * for freeing the allocated memory for the `http_response` structure using the 
 * `free_http_response` function.
 *
 * @param request The HTTP request to be sent.
 * @return Pointer to an `http_response` structure containing the response data.
 *         If the request fails, `NULL` is returned.
 */
http_response* fetch(http_request request);

/**
 * @brief Frees the memory allocated for an HTTP response.
 *
 * @param response Pointer to the http_response structure to be freed.
 */
void free_http_response(http_response* response);

#endif