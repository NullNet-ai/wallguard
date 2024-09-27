#ifndef __SERVER_REQUESTS_H__
#define __SERVER_REQUESTS_H__

#include <utils/common.h>
#include <platform/ident.h>

boolean_t wallmon_heartbeat(const char* server_url, platform_info* info);

#endif
