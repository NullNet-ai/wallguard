#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <sys/stat.h>

#define BIOS_ROM_BASE 0xF0000
#define BIOS_ROM_SIZE 0x10000

int main() {
    int fd;
    unsigned char *bios_mem;
    unsigned char *bios_rom;
    unsigned char *uuid_location = NULL;
    unsigned int smbios_offset;

    // Open /dev/mem to access physical memory
    fd = open("/dev/mem", O_RDONLY);
    if (fd < 0) {
        perror("Unable to open /dev/mem");
        return 1;
    }

    // Map the BIOS ROM area
    bios_mem = mmap(NULL, BIOS_ROM_SIZE, PROT_READ, MAP_SHARED, fd, BIOS_ROM_BASE);
    if (bios_mem == MAP_FAILED) {
        perror("Unable to map BIOS ROM");
        close(fd);
        return 1;
    }

    for (bios_rom = bios_mem; bios_rom < bios_mem + BIOS_ROM_SIZE - 4; bios_rom++) {
        if (memcmp(bios_rom, "_SM_", 4) == 0) {
            smbios_offset = bios_rom - bios_mem;
            printf("Found SMBIOS entry point at offset 0x%X\n", smbios_offset);
            uuid_location = bios_rom + 0x10;  // This offset may vary based on the SMBIOS version
            break;
        }
    }

    if (uuid_location) {
        printf("System UUID: ");
        for (int i = 0; i < 16; i++) {
            printf("%02X", uuid_location[i]);
            if (i == 3 || i == 5 || i == 7 || i == 9)
                printf("-");
        }
        printf("\n");
    } else {
        fprintf(stderr, "SMBIOS table not found.\n");
    }

    // Clean up
    munmap(bios_mem, BIOS_ROM_SIZE);
    close(fd);

    return 0;
}
