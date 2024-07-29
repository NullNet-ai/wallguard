#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <time.h>
#include <libgen.h>
#include <time.h>
#include <ifaddrs.h>
#include <netdb.h>

#define BOUNDARY_LENGTH 30

char* generateRandomBoundary();

// int main(int argc, char *argv[])
// {
//     char *boundary = generateRandomBoundary();
//     printf("Boundary: %s\n", boundary);
//     return 0;
// }

char* generateRandomBoundary() {
    // Define the valid characters for the boundary string
    const char valid_chars[] = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ'()+_,-./:=?!";
    int num_valid_chars = strlen(valid_chars);
    
    // Allocate memory for the boundary string (+1 for the null terminator)
    char *boundary = (char *)malloc(BOUNDARY_LENGTH + 2); // +2 for '-' prefix and null terminator
    
    if (boundary == NULL) {
        perror("Unable to allocate memory");
        exit(EXIT_FAILURE);
    }
    
    // Seed the random number generator
    srand(time(NULL));
    
    // Set the first character to '-'
    boundary[0] = '-';
    
    // Generate the random boundary string
    for (int i = 1; i < BOUNDARY_LENGTH + 1; i++) {
        boundary[i] = valid_chars[rand() % num_valid_chars];
    }
    
    // Null-terminate the string
    boundary[BOUNDARY_LENGTH + 1] = '\0';
    
    return boundary;
}