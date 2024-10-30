#ifndef _FILE_UTILS_H_
#define _FILE_UTILS_H_

#include <utils/common.h>

/**
 * @brief Checks if a file exists at the given path.
 *
 * @param path The path to the file to check.
 * @return `WM_TRUE` if the file exists, otherwise `WM_FALSE`.
 */
boolean_t file_exists(const char *path);

/**
 * @brief Checks if a directory exists.
 *
 * @param path The path of the directory to check.
 * @return `WM_TRUE` if the directory exists, `WM_FALSE` otherwise.
 */
boolean_t directory_exists(const char *path);

/**
 * Creates a directory path similar to "mkdir -p".
 *
 * @param path The directory path to create.
 * @return `WM_TRUE` if the directory was created successfully or already exists,
 *         `WM_FALSE` if there was an error during creation.
 */
boolean_t make_directory(const char *path);

/**
 * @brief Get the size of a file.
 *
 *
 * @param path The path to the file whose size is to be determined.
 * @return The size of the file in bytes, or `-1` if an error occurs.
 */
ssize_t file_size(const char *path);

/**
 * Extracts the filename from a given file path.
 *
 * @param path The full file path as a string.
 * @return A pointer to the filename within the given path.
 *         If the path does not contain any directory separators,
 *         the original path is returned.
 */
const char *filename(const char *path);

/**
 * Extracts the extension from a given file path.
 *
 * @param path The full file path as a string.
 * @return A pointer to the extension within the given path.
 *         If the path does not contain any dots,
 *         `NULL` is returned.
 */
const char *extension(const char * path);

/**
 * @brief Copies the content of the source file to the destination file.
 *
 * @param source The path to the source file.
 * @param destination The path to the destination file.
 * @return `WM_TRUE` if the file is successfully copied, `WM_FALSE` otherwise.
 */
boolean_t copy_file(const char *source, const char *destination);

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
 * @return `1` if the file was updated, `0` if not, and `-1` on error.
 */
int file_monitor_check(file_monitor *monitor);

/**
 * @brief Reads the content of a file and returns it as a dynamically allocated
 * buffer of `uint8_t`. The caller is responsible for freeing the memory.
 *
 * @param path The path to the file to be read.
 * @return A pointer to the file's content as a `uint8_t` buffer, or `NULL` if an error occurred.
 */
uint8_t *read_file_content(const char *path);

#endif  // FILE_UTILS_H
