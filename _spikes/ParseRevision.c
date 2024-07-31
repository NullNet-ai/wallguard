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

void getConfigRevisionTime(const char *filename);

int main(int argc, char *argv[])
{
    char *file_path = "./sampleConfig.xml";
    getConfigRevisionTime(file_path);
    return 0;
}

void getConfigRevisionTime(const char *filename) {
    FILE *file = fopen(filename, "r");
    if (file == NULL) {
        fprintf(stderr, "Could not open file %s\n", filename);
        return;
    }

    char buffer[1024];
    char *start;
    char *end;

    while (fgets(buffer, sizeof(buffer), file)) {
        start = strstr(buffer, "<revision>");
        if (start) {
            while (fgets(buffer, sizeof(buffer), file)) {
                start = strstr(buffer, "<time>");
                if (start) {
                    start += strlen("<time>");
                    end = strstr(start, "</time>");
                    if (end) {
                        *end = '\0';
                        printf("Revision Time: %s\n", start);
                        fclose(file);
                        return;
                    }
                }
            }
        }
    }

    fprintf(stderr, "Revision time tag not found\n");
    fclose(file);
}