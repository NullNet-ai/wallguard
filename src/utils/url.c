#include <utils/url.h>

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

static const char http_prefix[]  = "http://";
static const char https_prefix[] = "https://";

static boolean_t parse_protocol(const char* url, const char** hostname_start, boolean_t* tls) {
    if (strncmp(url, https_prefix, strlen(https_prefix)) == 0) {
        if (tls) {
            *tls = WM_TRUE;
        }
        *hostname_start = url + strlen(https_prefix);
    } else if (strncmp(url, http_prefix, strlen(http_prefix)) == 0) {
        if (tls) {
            *tls = WM_FALSE;
        }
        *hostname_start = url + strlen(http_prefix);
    } else {
        return WM_FALSE;
    }

    return WM_TRUE;
}

static const char* parse_port(const char* hostname_start, int* port, boolean_t tls) {
    const char* port_start = strchr(hostname_start, ':');

    if (!port_start) {
        if (port) {
            *port = tls ? 443 : 80;
        }

        port_start = strchr(hostname_start, '/');

        if (!port_start) {
            port_start = hostname_start + strlen(hostname_start);
        }

        return port_start;
    }

    if (port) {
        const char* port_number_start = port_start + 1;
        const char* port_end          = strchr(port_number_start, '/');

        if (port_end == NULL) {
            port_end = port_number_start + strlen(port_number_start);
        }

        char port_str[6];

        strncpy(port_str, port_number_start, port_end - port_number_start);
        port_str[port_end - port_number_start] = '\0';

        *port = atoi(port_str);
    }

    return port_start;
}

static boolean_t parse_path(const char* hostname_start, char* path, size_t path_maxsize) {
    if (!path) {
        return WM_TRUE;
    }

    if (path && path_maxsize == 0) {
        return WM_FALSE;
    }

    const char* path_start = strchr(hostname_start, '/');

    if (path_start != NULL && *path_start == '/') {
        size_t path_size = strlen(path_start);
        if (path_size >= path_maxsize) {
            return WM_FALSE;
        }

        strncpy(path, path_start, path_maxsize - 1);
        path[path_maxsize - 1] = '\0';
    } else {
        strcpy(path, "/");
    }

    return WM_TRUE;
}

static boolean_t parse_hostname(const char* hostname_start, size_t hostname_length, char* hostname,
                                size_t hostname_maxlen) {
    if (!hostname) {
        return WM_TRUE;
    }

    if (hostname_length >= hostname_maxlen || hostname_length == 0) {
        return WM_FALSE;
    }

    strncpy(hostname, hostname_start, hostname_length);
    hostname[hostname_length] = '\0';
    return WM_TRUE;
}

boolean_t parse_url(const char* url, char* hostname, size_t hostname_maxlen, char* path, size_t path_maxsize, int* port,
                    boolean_t* tls) {
    const char* hostname_start;
    if (!parse_protocol(url, &hostname_start, tls)) {
        return WM_FALSE;
    }

    const char* port_start = parse_port(hostname_start, port, *tls);

    if (!parse_hostname(hostname_start, port_start - hostname_start, hostname, hostname_maxlen)) {
        return WM_FALSE;
    }

    if (!parse_path(hostname_start, path, path_maxsize)) {
        return WM_FALSE;
    }

    return WM_TRUE;
}