#ifndef FILE_UTILS_H
#define FILE_UTILS_H

#include <utils/common.h>

/**
 * @brief Checks if a file exists at the given path.
 *
 * @param path The path to the file to check.
 * @return `WM_TRUE` if the file exists, otherwise `WM_FALSE`.
 */
boolean_t file_exists(const char *path);

/**
 * @brief Checks if all of the files in the given array exist.
 *
 * @param files An array of file paths to check.
 * @param count The number of files in the array.
 * @return `WM_TRUE` if all of the files exist, otherwise `WM_FALSE`.
 */
boolean_t files_exist(const char *files[], size_t count);

/**
 * @brief Struct representing a file monitor.
 */
typedef struct {
    const char *filepath;
    int         last_update;
} file_monitor;

/**
 * @brief Initializes a file monitor.
 *
 * @param monitor A pointer to the file_monitor structure to initialize.
 * @param filepath The path to the file to be monitored.
 * @return `WM_TRUE` on success, `WM_FALSE` on failure.
 */
boolean_t file_monitor_init(file_monitor *monitor, const char *filepath);

/**
 * @brief Checks if the monitored file has been updated.
 *
 * @param monitor A pointer to the file_monitor structure.
 * @return 1 if the file was updated, 0 if not, and -1 on error.
 */
int file_monitor_check(file_monitor *monitor);

#endif  // FILE_UTILS_H
