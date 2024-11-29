#include <net_capture/traffic_filter.h>
#include <logger/logger.h>
#include <utils/net.h>
#include <config.h>


static boolean_t generate_filter(const char* url, char* filter, size_t len) {
    char hostname[256] = {0};

    if (!parse_url(url, hostname, ARRAY_SIZE(hostname), NULL, 0, NULL, NULL)) {
        WALLMON_LOG_ERROR("Failed to parse URL: %s", url);
        return WM_FALSE;
    }

    char ipv4[INET_ADDRSTRLEN] = {0}, ipv6[INET6_ADDRSTRLEN] = {0};

    boolean_t has_v4 = resolve_hostname_v4(hostname, ipv4);
    boolean_t has_v6 = resolve_hostname_v6(hostname, ipv6);

    if (has_v4 && has_v6) {
        if ((size_t)snprintf(filter, len, "host %s or ip6 host %s", ipv4, ipv6) >= len) {
            WALLMON_LOG_ERROR("Filter output truncated: buffer size insufficient");
            return WM_FALSE;
        }
    } else if (has_v4) {
        if ((size_t)snprintf(filter, len, "host %s", ipv4) >= len) {
            WALLMON_LOG_ERROR("Filter output truncated: buffer size insufficient");
            return WM_FALSE;
        }
    } else if (has_v6) {
        if ((size_t)snprintf(filter, len, "ip6 host %s", ipv6) >= len) {
            WALLMON_LOG_ERROR("Filter output truncated: buffer size insufficient");
            return WM_FALSE;
        }
    } else {
        WALLMON_LOG_ERROR("Failed to resolve hostname: %s", hostname);
        return WM_FALSE;
    }

    return WM_TRUE;
}

boolean_t format_filter(char* code, size_t len) {
    char filter_server[256] = {0};
    char filter_monitor[256] = {0};

    if (!generate_filter(cfg_get_server_url(), filter_server, ARRAY_SIZE(filter_server)) ||
        !generate_filter(cfg_get_monitor_url(), filter_monitor, ARRAY_SIZE(filter_monitor))) {
        WALLMON_LOG_ERROR("Failed to generate filter code");
        return WM_FALSE;
    }

    if ((size_t)snprintf(code, len, "not (%s or %s)", filter_server, filter_monitor) >= len) {
        WALLMON_LOG_ERROR("Combined filter output truncated: buffer size insufficient");
        return WM_FALSE;
    }

    return WM_TRUE;
}

struct bpf_program* build_filter(pcap_t* handle, const char* filter) {
    struct bpf_program* bpf = W_MALLOC(sizeof(struct bpf_program));
    if (pcap_compile(handle, bpf, filter, 0, PCAP_NETMASK_UNKNOWN) == -1) {
        WALLMON_LOG_WARN("Failed to compile filter: '%s': %s", filter, pcap_geterr(handle));
        W_FREE(bpf);
        return NULL;
    }

    if (pcap_setfilter(handle, bpf) == -1) {
        WALLMON_LOG_ERROR("Faield to set filter: %s", pcap_geterr(handle));
        pcap_freecode(bpf);
        W_FREE(bpf);
        return NULL;
    }

    return bpf;
}
