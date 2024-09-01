#include <network/request.h>
#include <network/tcp.h>
#include <network/tls.h>

#include <stdlib.h>

struct request_handle {
    void*     hdl;
    boolean_t tls;
};

boolean_t request_start(request_handle** handle, const char* hostname, int port, boolean_t tls) {
    boolean_t res = WM_FALSE;

    (*handle)      = malloc(sizeof(request_handle));
    (*handle)->tls = tls;
    (*handle)->hdl = NULL;

    if (tls) {
        res = tls_start((tls_handle**)&((*handle)->hdl), hostname, port);
    } else {
        res = tcp_start((tcp_handle**)&((*handle)->hdl), hostname, port);
    }

    if (!res) {
        free(*handle);
        *handle = NULL;
    }

    return res;
}

ssize_t request_write(request_handle* handle, uint8_t* data, size_t len) {
    if (!handle) {
        return -1;
    }

    if (handle->tls) {
        return tls_write(handle->hdl, data, len);
    } else {
        return tcp_write(handle->hdl, data, len);
    }
}
ssize_t request_read(request_handle* handle, uint8_t* data, size_t len) {
    if (!handle) {
        return -1;
    }

    if (handle->tls) {
        return tls_read(handle->hdl, data, len);
    } else {
        return tcp_read(handle->hdl, data, len);
    }
}

void request_end(request_handle* handle) {
    if (handle->tls) {
        tls_end(handle->hdl);
    } else {
        tcp_end(handle->hdl);
    }

    free(handle);
}
