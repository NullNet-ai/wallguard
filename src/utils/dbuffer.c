#include <utils/dbuffer.h>
#include <string.h>

void buffer_init(buffer_t* buffer, size_t capacity) {
    buffer->capacity = capacity;
    buffer->offset   = 0;
    buffer->memory   = W_MALLOC(capacity);
}

void buffer_free(buffer_t* buffer) {
    buffer->capacity = 0;
    buffer->offset   = 0;

    W_FREE(buffer->memory);
    buffer->memory = NULL;
}

boolean_t buffer_write(buffer_t* buffer, const void* data, size_t len) {
    if (!buffer_can_write(buffer, len)) {
        return WM_FALSE;
    }

    memcpy(buffer->memory + buffer->offset, data, len);
    buffer->offset += len;

    return WM_TRUE;
}

boolean_t buffer_can_write(buffer_t* buffer, size_t len) {
    if (!buffer) {
        return WM_FALSE;
    }

    return buffer->offset + len <= buffer->capacity;
}

boolean_t buffer_is_empty(buffer_t* buffer) { return buffer->offset == 0; }

void buffer_clear(buffer_t* buffer) { buffer->offset = 0; }
