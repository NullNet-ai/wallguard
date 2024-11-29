#include <net_capture/dispatcher.h>
#include <logger/logger.h>
#include <config.h>
#include <string.h>
#include <utils/str.h>
#include <arpa/inet.h>

#ifdef WALLMON_RELEASE
// @TODO: Read and validate from the configuration
#define DBUFFER_SIZE 65536
#else
#define DBUFFER_SIZE 2048
#endif

struct dispatcher {
    buffer_t        buffer;
    pthread_mutex_t mutex;
    transmitter_t*  transmitter;
};

dispatcher_t* dispatcher_initialize(const char* public_key) {
    transmitter_t* transmitter = transmitter_initialize(public_key);

    if (!transmitter) {
        WALLMON_LOG_ERROR("Failed to intialize data transmitter");
        return NULL;
    }

    dispatcher_t* instance = W_MALLOC(sizeof(dispatcher_t));

    buffer_init(&instance->buffer, DBUFFER_SIZE);

    if (pthread_mutex_init(&instance->mutex, NULL) != 0) {
        WALLMON_LOG_ERROR("Failed to intialize mutex");
        transmitter_finalize(transmitter);
        buffer_free(&instance->buffer);
        W_FREE(instance);
        return NULL;
    }

    instance->transmitter = transmitter;

    return instance;
}

void dispatcher_finalize(dispatcher_t* instance) {
    buffer_free(&instance->buffer);
    pthread_mutex_destroy(&instance->mutex);
    transmitter_finalize(instance->transmitter);
    W_FREE(instance);
}

void dispatcher_write(dispatcher_t* instance, const char* device, struct timeval* time, const void* data, size_t len) {
    // Length of the device name + 1 byte as null terminator
    size_t device_len = strlen(device) + 1;

    // Total size is [len of the data] + [device len] + [4 bytes for timestamp]
    size_t total_len = len + device_len + sizeof(uint32_t);

    pthread_mutex_lock(&instance->mutex);
    buffer_t* buffer = &instance->buffer;

    /**
     * Check if buffer can accomodate chunk + device name.
     * If buffer is full, push the data to the transmitter and clear the buffer
     */
    if (!buffer_can_write(buffer, total_len)) {
        transmitter_send(instance->transmitter, buffer->memory, buffer->offset);
        buffer_clear(buffer);
    }

    /**
     * If the buffer is empty, prepand the system's UUID
     * `cfg_get_system_uuid` is expected to return a valid UUID, be sure to validate it before.
     */
    if (buffer_is_empty(buffer)) {
        const char* uuid      = cfg_get_system_uuid();
        uint8_t     bytes[16] = {0};
        uuid_to_bytes(uuid, bytes);
        buffer_write(buffer, bytes, ARRAY_SIZE(bytes));
    }

    /**
     * Chunk's structure
     * [ifname, device_len][timestamp, 4 bytes][packet info, `len` bytes]
     */
    {
        // Write interface
        buffer_write(buffer, device, device_len);

        // Write timestamp in BE
        uint32_t timestamp = htonl((uint32_t)time->tv_sec);
        buffer_write(buffer, &timestamp, sizeof(uint32_t));

        // Write packet info
        buffer_write(buffer, data, len);
    }

    pthread_mutex_unlock(&instance->mutex);
}
