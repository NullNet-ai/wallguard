#include "wsignal.h"

#include <logger/logger.h>

#include <errno.h>
#include <signal.h>
#include <string.h>
#include <stdatomic.h>

typedef _Atomic(boolean_t) atomic_boolean_t;

static atomic_boolean_t running_flag = ATOMIC_VAR_INIT(WM_TRUE);

static void handle_signal(int signal) {
    WALLMON_LOG_INFO("Received signal: %s", strsignal(signal));

    if (signal == SIGINT) {
        atomic_store(&running_flag, WM_FALSE);
    }
}

boolean_t wallmon_is_running() { return atomic_load(&running_flag); }

boolean_t wallmon_setup_signal_handler() {
    if (signal(SIGINT, handle_signal) == SIG_ERR) {
        WALLMON_LOG_WARN("Failed to setup SIGINT handler: %s", strerror(errno));
        return WM_FALSE;
    }

    return WM_TRUE;
}
