#ifndef _WSIGNAL_H_
#define _WSIGNAL_H_

#include <utils/common.h>

/**
 * @brief Checks if the running flag is set.
 *
 * @return `WM_TRUE` if wallmon should continue running, `WM_FALSE` if it should terminate.
 */
boolean_t wallmon_is_running();

/**
 * @brief Sets up custom signal handlers for Wallmon.
 *
 * @return `WM_TRUE` if the signal handlers were successfully set up, `WM_FALSE` otherwise.
 */
boolean_t wallmon_setup_signal_handler();

#endif
