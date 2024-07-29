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

char* executeCommand(const char* command);

int main(int argc, char *argv[])
{
    char *command = "ls -l";
    char *result = executeCommand(command);
    printf("Result: %s\n", result);

    return 0;
}
char* executeCommand(const char* command) {

    // printf("Executing executeCommandWithReturn: %s\n", command);

    char* result = NULL;
    char buffer[128];
    size_t resultSize = 1;
    FILE* pipe = popen(command, "r");

    if (!pipe) {
        return NULL;
    }

    result = (char*)malloc(resultSize);
    if (!result) {
        pclose(pipe);
        return NULL;
    }

    result[0] = '\0';

    while (fgets(buffer, sizeof(buffer), pipe) != NULL) {
        resultSize += strlen(buffer);
        result = (char*)realloc(result, resultSize);
        if (!result) {
            pclose(pipe);
            return NULL;
        }
        strcat(result, buffer);
    }

    pclose(pipe);
    return result;
}