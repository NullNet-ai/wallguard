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
#include "RandomBoundry.c"

// Rest of the code...

#define BUFFER_SIZE 1024
#define PORT 3000
#define SERVER "172.16.70.71" //"127.0.0.1" 

int send_file(const char *file_path, const char *uuid, const char *ou_id);

int main(int argc, char *argv[])
{
    return 0;
}

int send_file(const char *file_path, const char *uuid, const char *ou_id)
{
    int sockfd;
    struct sockaddr_in server_addr;
    char buffer[BUFFER_SIZE];
    FILE *file;
    int bytes_read;
    long file_size;
    char *temp_file_path = strdup(file_path);
    char *base_file_name = basename(temp_file_path);
    // Generate a random boundary string
    char *boundary = generateRandomBoundary();
    printf("Generated Boundary: %s\n", boundary);

    // Separate the filename and the extension
    char *dot = strrchr(base_file_name, '.');
    char *name_without_ext;
    char *ext = "";

    if (dot) {
        *dot = '\0'; // Null-terminate the name part
        name_without_ext = base_file_name;
        ext = dot + 1; // Point to the extension part
    } else {
        name_without_ext = base_file_name;
    }

    // Get current UTC date and time
    time_t now = time(NULL);
    struct tm *tm = gmtime(&now);
    char date_time[20]; // Buffer to hold the date and time string
    strftime(date_time, sizeof(date_time), "%Y%m%d%H%M%S", tm); // Format: YYYYMMDDHHMMSS

    // Append date and time to the filename, then append the extension
    char file_name[256]; // Buffer to hold the new filename
    snprintf(file_name, sizeof(file_name), "%s_%s_%s_%s.%s", name_without_ext, ou_id, uuid, date_time, ext);

    // Create socket
    if ((sockfd = socket(AF_INET, SOCK_STREAM, 0)) < 0)
    {
        perror("Socket creation failed");
        return 1;
    }

    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(PORT);
    inet_pton(AF_INET, SERVER, &server_addr.sin_addr);

    // Connect to the server
    if (connect(sockfd, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0)
    {
        perror("Connection failed");
        close(sockfd);
        return 1;
    }
    else
    {
        printf("Connected to server\n");
    }

    // Open file
    file = fopen(file_path, "rb");
    if (!file)
    {
        perror("File opening failed");
        close(sockfd);
        return 1;
    }
    else
    {
        printf("Opened file: %s\n", file_path);
    }

    // Determine file size
    fseek(file, 0, SEEK_END);
    file_size = ftell(file);
    printf("File size: %ld bytes\n", file_size);
    fseek(file, 0, SEEK_SET);

    char header[1024];
    sprintf(header, "POST /in/%s HTTP/1.1\r\n"
        "User-Agent: myPfsense\r\n"
        "Accept: */*\r\n"
        "Host: %s:%d\r\n"
        "Content-Type: multipart/form-data; boundary=%s\r\n",
        ou_id, SERVER, PORT, boundary);

    // printf("%s\n", header);
    int header_len = strlen(header);
    // printf("Header size: %d\n", header_len);

    char content[1024];

    sprintf(content,
        "--%s\r\n"
        "Content-Disposition: form-data; name=\"file\"; filename=\"%s\"\r\n"
        "Content-Type: application/octet-stream\r\n"
        "\r\n",
        boundary, file_name);

    // printf("%s\n", content);
    int content_len = strlen(content);
    printf("Content size: %d\n", content_len);


    char closing[1024];
    sprintf(closing,
        "\r\n--%s--"
        , boundary);

    // printf("%s\n", closing);
    int closing_len = strlen(closing);
    // printf("Closing size: %d\n", closing_len);
    // printf("\n");


    snprintf(buffer, sizeof(buffer),
        "POST /in/%s HTTP/1.1\r\n"
        "User-Agent: myPfsense\r\n"
        "Accept: */*\r\n"
        "Host: %s:%d\r\n"
        "Content-Type: multipart/form-data; boundary=%s\r\n",
        ou_id,SERVER, PORT, boundary);
    printf("%s\n", buffer);
    send(sockfd, buffer, strlen(buffer), 0);

    // Create form-data content
    snprintf(buffer, sizeof(buffer),
        "Content-Length: %ld\r\n"
        "\r\n",
                file_size + content_len + closing_len);
    // printf("%s\n", buffer);
    send(sockfd, buffer, strlen(buffer), 0);
    printf("Total length: %ld\n", file_size + content_len + closing_len);

    // Create form-data content
    snprintf(buffer, sizeof(buffer),
        "--%s\r\n"
        "Content-Disposition: form-data; name=\"file\"; filename=\"%s\"\r\n"
        "Content-Type: application/octet-stream\r\n"
        "\r\n",
        boundary, file_name);
    // printf("%s\n", buffer);
    send(sockfd, buffer, strlen(buffer), 0);

    // Read file and send
    while ((bytes_read = fread(buffer, 1, BUFFER_SIZE, file)) > 0)
    {
        send(sockfd, buffer, bytes_read, 0);
    }
    
    // End of form-data
    snprintf(buffer, sizeof(buffer), "\r\n--%s--\r\n", boundary);
    // printf("%s\n", buffer);
    send(sockfd, buffer, strlen(buffer), 0);

    printf("File %s sent\n", file_path);

    // Close file and socket
    fclose(file);
    close(sockfd);
    free(temp_file_path);
    return 0;
}