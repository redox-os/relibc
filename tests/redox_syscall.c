#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <time.h>
#include <errno.h>

int main() {
    const char *disk_path = "/scheme/disk.pci-0000-00-06.0-nvme/1";

    printf("Opening '%s' for syscall benchmark...\n", disk_path);
    int fd = open(disk_path, O_RDWR);
    if (fd < 0) {
        printf("Failed to open %s: %s\n", disk_path, strerror(errno));
        return 1;
    }

    uint8_t *buf = aligned_alloc(4096, 4096);
    if (!buf) {
        printf("Failed to allocate buffer\n");
        return 1;
    }
    memset(buf, 0, 4096);

    int total_ops = 32000;
    
    printf("\nStarting Syscall Write Benchmark...\n");
    fflush(stdout);

    struct timespec start, end;
    
    if (lseek(fd, 0, SEEK_SET) < 0) {
        printf("lseek error: %s\n", strerror(errno));
        return 1;
    }

    clock_gettime(CLOCK_MONOTONIC, &start);

    for (int i = 0; i < total_ops; i++) {
        if (write(fd, buf, 4096) != 4096) {
            printf("write error at index %d: %s\n", i, strerror(errno));
            return 1;
        }
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    double duration = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    double bytes_processed = (double)total_ops * 4096.0;
    double mbps = (bytes_processed / 1024.0 / 1024.0) / duration;

    printf("Syscall Write Benchmark: %d ops in %.3f seconds (%.2f MB/s)\n", total_ops, duration, mbps);

    printf("\nStarting Syscall Read Benchmark...\n");
    fflush(stdout);

    if (lseek(fd, 0, SEEK_SET) < 0) {
        printf("lseek error: %s\n", strerror(errno));
        return 1;
    }

    clock_gettime(CLOCK_MONOTONIC, &start);

    for (int i = 0; i < total_ops; i++) {
        if (read(fd, buf, 4096) != 4096) {
            printf("read error at index %d: %s\n", i, strerror(errno));
            return 1;
        }
    }

    clock_gettime(CLOCK_MONOTONIC, &end);
    duration = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    mbps = (bytes_processed / 1024.0 / 1024.0) / duration;

    printf("Syscall Read Benchmark: %d ops in %.3f seconds (%.2f MB/s)\n", total_ops, duration, mbps);

    close(fd);
    free(buf);

    printf("\nTests finished successfully.\n");
    return 0;
}
