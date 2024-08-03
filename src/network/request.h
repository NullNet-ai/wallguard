#ifndef NET_REQUEST_H
#define NET_REQUEST_H

#include <utils/common.h>

boolean_t https_request(const char* hostname, int port, void* data, size_t len);

boolean_t http_request(const char* hostname, int port, void* data, size_t len);

#endif