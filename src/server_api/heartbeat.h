#ifndef _SERVER_API_HEARTBEAT_H_
#define _SERVER_API_HEARTBEAT_H_

#include <utils/common.h>
#include <platform/ident.h>

typedef enum {
    SACTION_REUPLOAD = 0,
    SACTION_NONE,
} server_action;

/**
 * @brief Sends a heartbeat signal to the central server to indicate that the system is still active.
 *
 * @param server_url The URL of the server where the registration request is sent.
 * @return `WM_TRUE` if the request is successful, `WM_FALSE` otherwise.
 *
 * @note The `platform_info` structure should be properly populated with the necessary platform details before calling
 * this function.
 */
boolean_t heartbeat_request(const char* server_url, platform_info* info, server_action* action);

#endif
