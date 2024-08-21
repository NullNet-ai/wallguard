#ifndef UTILS_URL_H
#define UTILS_URL_H

#include <utils/common.h>

boolean_t parse_url(const char* url, char* hostname, size_t hostname_maxlen, char* path, size_t path_maxsize, int* port,
                    boolean_t* tls);

#endif
