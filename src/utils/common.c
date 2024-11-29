#include <utils/common.h>
#include <logger/logger.h>

#define ASSERT_PTR_NOT_NULL(PTR)            \
    if (PTR == NULL) {                      \
        WALLMON_LOG_ERROR("Out of memory"); \
        exit(EXIT_FAILURE);                 \
    }

#define ASSET_SIZE_NON_ZERO(SIZE)                         \
    if (SIZE == 0) {                                      \
        WALLMON_LOG_ERROR("Attempt to allocate 0 bytes"); \
        exit(EXIT_FAILURE);                               \
    }

void* __wallmon_malloc(size_t size) {
    ASSET_SIZE_NON_ZERO(size);
    void* ptr = malloc(size);
    ASSERT_PTR_NOT_NULL(ptr);
    return ptr;
}

void* __wallmon_calloc(size_t nmem, size_t size) {
    ASSET_SIZE_NON_ZERO(size);
    void* ptr = calloc(nmem, size);
    ASSERT_PTR_NOT_NULL(ptr);
    return ptr;
}

void* __wallmon_realloc(void* ptr, size_t size) {
    ASSET_SIZE_NON_ZERO(size);
    void* ptr_new = realloc(ptr, size);
    ASSERT_PTR_NOT_NULL(ptr_new);
    return ptr_new;
}
