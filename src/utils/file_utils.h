#ifndef FILE_UTILS_H
#define FILE_UTILS_H

/**
 * @brief Checks if a file exists at the given path.
 *
 * @param path The path to the file to check.
 * @return 1 if the file exists, otherwise 0.
 */
int file_exists(const char *path);

/**
 * @brief Checks if all of the files in the given array exist.
 *
 * @param files An array of file paths to check.
 * @param count The number of files in the array.
 * @return 1 if all of the files exist, otherwise 0.
 */
int files_exist(const char *files[], int count);

#endif // FILE_UTILS_H
