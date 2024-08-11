#include <utils/url.h>

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

static const char http_prefix[]  = "http://";
static const char https_prefix[] = "https://";

boolean_t parse_url(const char* url, char* hostname, char* path, int* port, boolean_t* tls) {
    const char* hostname_start = NULL;

    if (strncmp(url, https_prefix, strlen(https_prefix)) == 0) {
        if (tls) {
            *tls = WM_TRUE;
        }
        hostname_start = url + strlen(https_prefix);
    } else if (strncmp(url, http_prefix, strlen(http_prefix)) == 0) {
        if (tls) {
            *tls = WM_FALSE;
        }
        hostname_start = url + strlen(http_prefix);
    } else {
        return WM_FALSE;
    }

    size_t hostname_length;

    const char* port_start = strchr(hostname_start, ':');
    const char* path_start = strchr(hostname_start, '/');

    if (port_start != NULL) {
        hostname_length = port_start - hostname_start;

        const char* port_number_start = port_start + 1;
        const char* port_end          = strchr(port_number_start, '/');

        if (port_end == NULL) {
            port_end = port_number_start + strlen(port_number_start);
        }

        char port_str[6];

        strncpy(port_str, port_number_start, port_end - port_number_start);
        port_str[port_end - port_number_start] = '\0';

        if (port) {
            *port = atoi(port_str);
        }
    } else {
        if (port) {
            *port = (*tls) ? 443 : 80;
        }

        port_start = strchr(hostname_start, '/');

        if (port_start == NULL) {
            port_start = hostname_start + strlen(hostname_start);
        }

        hostname_length = port_start - hostname_start;
    }

    if (path) {
        if (path_start != NULL && *path_start == '/') {
            strncpy(path, path_start, strlen(path_start));
        } else {
            strcpy(path, "/");
        }
    }

    if (hostname) {
        strncpy(hostname, hostname_start, hostname_length);
        hostname[hostname_length] = '\0';
    }

    return WM_TRUE;
}