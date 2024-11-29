#include <net_capture/dispatcher.h>
#include <logger/logger.h>
#include <config.h>
#include <string.h>

#include <net/if.h>
STATIC_ASSERT(IF_NAMESIZE == 16, "IF_NAMESIZE is not 16 bytes on the current platform.");

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

void dispatcher_write(dispatcher_t* instance, const char* device, const void* data, size_t len) {
    pthread_mutex_lock(&instance->mutex);
    buffer_t* buffer = &instance->buffer;

    /**
     * Check if buffer can accomodate chunk + device name.
     * If buffer is full, push the data to the transmitter and clear the buffer
     */
    if (!buffer_can_write(buffer, len + IF_NAMESIZE)) {
        transmitter_send(instance->transmitter, buffer->memory, buffer->offset);
        buffer_clear(buffer);
    }

    /**
     * If the buffer is empty, prepand the system's UUID
     * `cfg_get_system_uuid` is expected to return a valid UUID, be sure to validate it before.
     */
    if (buffer_is_empty(buffer)) {
        const char* uuid = cfg_get_system_uuid();
        // @TODO: Convert to 16 bytes binary format
        buffer_write(buffer, uuid, 36);
    }

    /**
     * Chunk's structure
     * [ifname, 16 bytes][packet info, `len` bytes]
     */

    char ifname[IF_NAMESIZE] = {0};

    size_t devlen = strlen(device);
    memcpy(ifname, device, devlen > IF_NAMESIZE ? IF_NAMESIZE : devlen);

    buffer_write(buffer, ifname, IF_NAMESIZE);
    buffer_write(buffer, data, len);

    pthread_mutex_unlock(&instance->mutex);
}
