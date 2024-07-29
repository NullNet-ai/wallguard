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

int rec_file(const char *uuid, const char *ou_id);

int main(int argc, char *argv[])
{
    return 0;
}

int rec_file(const char *uuid, const char *ou_id){
    int sockfd, portno = 3000;
    struct sockaddr_in serv_addr;
    struct hostent *server;
    char buffer[1024];
    const char *hostname = "localhost";
    char *path[1024];
    snprintf(path, sizeof(path), "/out/%s/%s", ou_id, uuid);
    char *outfilename[1024];
    snprintf(outfilename, sizeof(outfilename), 
        "config_%s.xml", uuid);

    FILE *fp;

    sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0)
        error("ERROR opening socket");

    server = gethostbyname(hostname);
    if (server == NULL)
    {
        fprintf(stderr, "ERROR, no such host\n");
        exit(0);
    }

    bzero((char *)&serv_addr, sizeof(serv_addr));
    serv_addr.sin_family = AF_INET;
    bcopy((char *)server->h_addr, (char *)&serv_addr.sin_addr.s_addr, server->h_length);
    serv_addr.sin_port = htons(portno);

    if (connect(sockfd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0)
        error("ERROR connecting");

    snprintf(buffer, sizeof(buffer), "GET %s HTTP/1.1\r\nHost: %s\r\nConnection: close\r\n\r\n", path, hostname);
    if (write(sockfd, buffer, strlen(buffer)) < 0)
        error("ERROR writing to socket");

    fp = fopen(outfilename, "wb");
    if (fp == NULL)
    {
        perror("File open failed");
        return 1;
    }

    bzero(buffer, sizeof(buffer));
    int bytes_read;
    int header_passed = 0;
    while ((bytes_read = read(sockfd, buffer, sizeof(buffer) - 1)) > 0)
    {
        buffer[bytes_read] = '\0';
        if (!header_passed)
        {
            char *header_end = strstr(buffer, "\r\n\r\n");
            if (header_end != NULL)
            {
                fwrite(header_end + 4, 1, bytes_read - (header_end + 4 - buffer), fp);
                header_passed = 1;
            }
        }
        else
        {
            fwrite(buffer, 1, bytes_read, fp);
        }
    }

    if (bytes_read < 0)
        error("ERROR reading from socket");

    close(sockfd);
    fclose(fp);
    return 0;
}