#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <sys/statvfs.h>

void test_statvfs(const char *path) {
    struct statvfs buf;

    printf("Testing statvfs on: %s\n", path);

    if (statvfs(path, &buf) != 0) {
        perror("statvfs failed");
        exit(EXIT_FAILURE);
    }

    printf("  Filesystem block size: %lu\n", buf.f_bsize);
    printf("  Fragment size:         %lu\n", buf.f_frsize);
    printf("  Total blocks:          %lu\n", buf.f_blocks);
    printf("  Free blocks:           %lu\n", buf.f_bfree);
    printf("  Available blocks:      %lu\n", buf.f_bavail);
    printf("  Total inodes:          %lu\n", buf.f_files);
    printf("  Free inodes:           %lu\n", buf.f_ffree);

    if (buf.f_bsize == 0 || buf.f_frsize == 0) {
        fprintf(stderr, "ERROR: Block size or fragment size is zero.\n");
        exit(EXIT_FAILURE);
    }

    if (buf.f_blocks == 0) {
        fprintf(stderr, "ERROR: Total number of blocks is zero.\n");
        exit(EXIT_FAILURE);
    }

    printf("Test passed: statvfs returned valid values.\n");
}

int main(int argc, char *argv[]) {
    const char *test_path = "/";

    if (argc > 1) {
        test_path = argv[1];
    }

    test_statvfs(test_path);

    return 0;
}
