#ifndef UTILS_NET_H
#define UTILS_NET_H

#include <network/http.h>
#include <network/request.h>

/**
 * @brief Parses a raw HTTP response string and populates an `http_response` structure.
 *
 * @param raw_response  The raw HTTP response string to be parsed.
 * @param response_len  The length of the raw HTTP response string.
 *
 * @return A pointer to an `http_response` structure containing the parsed response, or `NULL`.
 */
http_response* parse_response(char* raw_response, size_t response_len);

/**
 * @brief Frees the memory allocated for the response.
 *
 * @param response The response structure to free.
 */
void release_response(http_response* response);

/**
 * @brief Reads the full response from a request handle into a dynamically allocated buffer.
 *
 * @param handle    The request handle from which to read data.
 * @param data      A pointer to a pointer that will be set to the dynamically allocated buffer
 *                  containing the response data. The caller is responsible for freeing this buffer.
 * @param len       A pointer to a variable that will be set to the length of the data read.
 * @param terminate If `WM_TRUE`, the data will be null-terminated.
 *
 * @return `WM_TRUE` if the data is successfully read, `WM_FALSE` on error (e.g., memory allocation failure
 *         or read error). In the case of an error, the buffer is freed and `data` is set to `NULL`.
 */
boolean_t read_response_full(request_handle* handle, uint8_t** data, size_t* len, boolean_t terminate);
#endif