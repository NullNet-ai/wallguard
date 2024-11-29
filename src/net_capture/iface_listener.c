#include <net_capture/iface_listener.h>
#include <net_capture/traffic_filter.h>
#include <utils/wsignal.h>
#include <utils/str.h>

#ifndef __USE_MISC
#define __USE_MISC
#endif

#include <netinet/if_ether.h>
#include <netinet/ip.h>
#include <netinet/tcp.h>
#include <netinet/udp.h>

#include <config.h>
#include <logger/logger.h>

#include <string.h>

void* iface_listener_routine(void* arg) {
    iface_listener_info_t* arguments = (iface_listener_info_t*)arg;

    pcap_t*          handle   = arguments->handle;
    const char*      device   = arguments->device;
    write_callback_t callback = arguments->callback;

    while (wallmon_is_running()) {
        struct pcap_pkthdr packet_header;
        const void*        packet = pcap_next(handle, &packet_header);

        if (packet == NULL) {
            // pcap_next() returned NULL during live capture, which can occur for a few reasons:
            // 1. No packets passed the capture filter, so none were available for retrieval.
            // 2. The packet buffer timeout expired before any packets arrived (on systems with this setting).
            // 3. If the capture device is in non-blocking mode, no packets were available at this moment.
            //
            // Since these are all normal scenarios in a live capture and do not necessarily indicate an error,
            // we continue to the next iteration, allowing the loop to keep attempting packet retrieval.
            continue;
        }

        struct ether_header* eth_hdr = (struct ether_header*)packet;
        if (ntohs(eth_hdr->ether_type) != ETHERTYPE_IP) {
            continue;
        }

        struct ip* ip_hdr = (struct ip*)(packet + sizeof(struct ether_header));
        if (ip_hdr->ip_p != IPPROTO_TCP && ip_hdr->ip_p != IPPROTO_UDP) {
            continue;
        }

        size_t data_len = sizeof(struct ether_header) + ip_hdr->ip_hl * 4;

        if (ip_hdr->ip_p == IPPROTO_TCP) {
            data_len += sizeof(struct tcphdr);
        } else {
            data_len += sizeof(struct udphdr);
        }

        callback(device, packet, data_len);
    }

    return NULL;
}

static void apply_filter(iface_listener_info_t* info) {
    char filter[256] = {0};
    if (!format_filter(filter, ARRAY_SIZE(filter))) {
        WALLMON_LOG_WARN("Failed to format filter");
        return;
    }

    const char* iface = cfg_get_netwrok_interface();

    if (iface && strcmp(iface, info->device) != 0) {
        // We're only interested in the interface that is used for server communication
        return;
    }

    struct bpf_program* bpf = build_filter(info->handle, filter);
    if (bpf) {
        info->filter = bpf;
        WALLMON_LOG_INFO("Successfully applied filter '%s' to interface %s", filter, info->device);
    }
}

llist_t* build_listeners_list(size_t buffer_size, write_callback_t callback) {
    char       errbuf[PCAP_ERRBUF_SIZE] = {0};
    pcap_if_t* if_list                  = NULL;

    if (pcap_findalldevs(&if_list, errbuf) == PCAP_ERROR) {
        WALLMON_LOG_ERROR("Failed to find interface devices: %s", errbuf);
        return NULL;
    }

    if (if_list == NULL) {
        WALLMON_LOG_ERROR("No interfaces has been found");
        return NULL;
    }

    llist_t* if_handles = NULL;

    for (pcap_if_t* iface = if_list; iface != NULL; iface = iface->next) {
        if (iface->addresses == NULL || iface->flags & PCAP_IF_LOOPBACK) {
            continue;
        }

        pcap_t* handle = pcap_open_live(iface->name, buffer_size, 1, 1000, errbuf);

        if (!handle) {
            continue;
        }

        // @TODO:
        // For now pick ONLY DLT_EN10MB datalink
        if (pcap_datalink(handle) != DLT_EN10MB) {
            pcap_close(handle);
            continue;
        }

        iface_listener_info_t* listener = W_MALLOC(sizeof(iface_listener_info_t));

        listener->handle   = handle;
        listener->device   = string_copy(iface->name);
        listener->callback = callback;
        listener->filter   = NULL;

        apply_filter(listener);

        if (if_handles) {
            ll_push_back(&if_handles, listener);
        } else {
            if_handles = ll_create_node(listener);
        }
    }

    pcap_freealldevs(if_list);
    return if_handles;
}

void free_listeners_list(llist_t* list) {
    LL_FOREACH(list, element) {
        iface_listener_info_t* info = (iface_listener_info_t*)element->data;

        if (info->filter) {
            pcap_freecode(info->filter);
            W_FREE(info->filter);
        }

        pcap_close(info->handle);
        W_FREE((void*)info->device);
        W_FREE(info);
    }

    ll_free(list);
}
