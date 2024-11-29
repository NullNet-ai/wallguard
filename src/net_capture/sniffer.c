#include <logger/logger.h>
#include <net_capture/iface_listener.h>
#include <net_capture/dispatcher.h>
#include <net_capture/sniffer.h>
#include <utils/linked_list.h>
#include <pthread.h>
#include <string.h>

#define PCAP_BUFFER_SIZE 32768

static dispatcher_t* disp = NULL;

struct sniffer {
    llist_t* listeners;  // List of `iface_listener_info_t`
};

static void lwphdr_clbck(const char* device, const void* data, size_t len) {
    dispatcher_write(disp, device, data, len);
}

sniffer_t* sniffer_initialize(const char* public_key) {
    disp = dispatcher_initialize(public_key);
    if (!disp) {
        WALLMON_LOG_ERROR("Failed to initialize the data dispatcher");
        return NULL;
    }

    llist_t* if_handles = build_listeners_list(PCAP_BUFFER_SIZE, lwphdr_clbck);

    if (if_handles == NULL) {
        WALLMON_LOG_ERROR("There are no interfaces that match the supported criteria.");
        return NULL;
    }

    sniffer_t* instance = W_MALLOC(sizeof(sniffer_t));
    instance->listeners = if_handles;
    return instance;
}

void sniffer_mainloop(sniffer_t* sniffer) {
    size_t if_len = ll_length(sniffer->listeners);
    if (if_len == 0) return;

    pthread_t* threads = W_MALLOC(if_len * sizeof(pthread_t));

    size_t index = 0;
    LL_FOREACH(sniffer->listeners, element) {
        int result = pthread_create(&threads[index], NULL, iface_listener_routine, element->data);
        if (result != 0) {
            WALLMON_LOG_ERROR("Failed to spawn listeners threads: %s", strerror(result));
            for (size_t i = 0; i < index; ++i) {
                pthread_cancel(threads[i]);
                pthread_join(threads[i], NULL);
            }

            W_FREE(threads);
            return;
        }
        ++index;
    }

    for (size_t i = 0; i < if_len; ++i) {
        pthread_join(threads[i], NULL);
    }

    W_FREE(threads);
    return;
}

void sniffer_finalize(sniffer_t* sniffer) {
    free_listeners_list(sniffer->listeners);
    W_FREE(sniffer);

    dispatcher_finalize(disp);
}
