#ifndef _CONFIG_H_
#define _CONFIG_H_

void cfg_init(const char* filename);
void cfg_deinit(void);

void cfg_validate(void);

const char* cfg_get_server_url();
const char* cfg_get_instance_name();

const char* cfg_get_api_key();
const char* cfg_get_access_token();

const char* cfg_get_netwrok_interface();
const char* cfg_get_system_uuid();

const char* cfg_get_platform();
int         cfg_get_heartbeat_interval();

#endif
