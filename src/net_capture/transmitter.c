#include <net_capture/transmitter.h>
#include <utils/net.h>
#include <logger/logger.h>
#include <config.h>
#include <zmq.h>

#include <string.h>

struct transmitter {
    void* context;
    void* socket;
};

static boolean_t configure_curve(void* socket, const char* server_key) {
    char public_key[41] = {0}, secret_key[41] = {0};
    if (zmq_curve_keypair(public_key, secret_key) != 0) {
        WALLMON_LOG_ERROR("Failed to generate curve keypair: %s", zmq_strerror(zmq_errno()));
        return WM_FALSE;
    }

    if (zmq_setsockopt(socket, ZMQ_CURVE_SERVERKEY, server_key, strlen(server_key)) != 0) {
        WALLMON_LOG_ERROR("Failed to set server key to ZMQ socket: %s", zmq_strerror(zmq_errno()));
        return WM_FALSE;
    }

    if (zmq_setsockopt(socket, ZMQ_CURVE_PUBLICKEY, public_key, strlen(public_key)) != 0) {
        WALLMON_LOG_ERROR("Failed to set public key to ZMQ socket: %s", zmq_strerror(zmq_errno()));
        return WM_FALSE;
    }

    if (zmq_setsockopt(socket, ZMQ_CURVE_SECRETKEY, secret_key, strlen(secret_key)) != 0) {
        WALLMON_LOG_ERROR("Failed to set secret key to ZMQ socket: %s", zmq_strerror(zmq_errno()));
        return WM_FALSE;
    }

    return WM_TRUE;
}

transmitter_t* transmitter_initialize(const char* server_key) {
    void* context = zmq_ctx_new();

    if (!context) {
        WALLMON_LOG_ERROR("Failed to create zmq context: %s", zmq_strerror(zmq_errno()));
        return NULL;
    }

    void* socket = zmq_socket(context, ZMQ_PUSH);
    if (!socket) {
        WALLMON_LOG_ERROR("Failed to create zmq socket: %s", zmq_strerror(zmq_errno()));
        zmq_ctx_destroy(context);
        return NULL;
    }

    if (!configure_curve(socket, server_key)) {
        zmq_close(socket);
        zmq_ctx_destroy(context);
        return NULL;
    }

    if (zmq_connect(socket, cfg_get_monitor_url()) != 0) {
        WALLMON_LOG_ERROR("Failed to connect to %s : %s", cfg_get_monitor_url(), zmq_strerror(zmq_errno()));

        zmq_close(socket);
        zmq_ctx_destroy(context);
        return NULL;
    }

    int timeval = 5000;
    if (zmq_setsockopt(socket, ZMQ_SNDTIMEO, &timeval, sizeof(int)) != 0) {
        WALLMON_LOG_WARN("Could not set timeout to transmitter socket: %s", zmq_strerror(zmq_errno()));
    }

    int linger = 0;
    if (zmq_setsockopt(socket, ZMQ_LINGER, &linger, sizeof(int)) != 0) {
        WALLMON_LOG_WARN("Could not set linger option: %s", zmq_strerror(zmq_errno()));
    }

    transmitter_t* handle = W_MALLOC(sizeof(transmitter_t));

    handle->context = context;
    handle->socket  = socket;

    return handle;
}

boolean_t transmitter_send(transmitter_t* handle, void* data, size_t len) {
    int result = zmq_send(handle->socket, data, len, 0);

    if (result == -1) {
        // Probably High Water Mark has been reached and the ZMQ queue is full.
        WALLMON_LOG_ERROR("Failed to send zmq message: %s", zmq_strerror(zmq_errno()));
        return WM_FALSE;
    }

    return WM_TRUE;
}

void transmitter_finalize(transmitter_t* handle) {
    zmq_close(handle->socket);
    zmq_ctx_term(handle->context);
    W_FREE(handle);
}
