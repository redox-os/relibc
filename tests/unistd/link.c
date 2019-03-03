#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <unistd.h>

#include "test_helpers.h"

int main(void) {
    printf("sizeof(struct stat): %ld\n", sizeof(struct stat));

    struct stat buf;

    // Stat for the inode
    if (stat("unistd/link.c", &buf)) {
        perror("stat");
        exit(EXIT_FAILURE);
    }
    unsigned long inode = buf.st_ino;
    printf("%ld\n", inode);

    // Create the link
    if (link("unistd/link.c", "link.out")) {
        perror("link");
        exit(EXIT_FAILURE);
    }

    // Make sure inodes match
    if (stat("link.out", &buf)) {
        perror("stat");
    }
    printf("%ld\n", inode);
    printf("%ld\n", buf.st_ino);
    if (inode != buf.st_ino) {
        puts("Created file is not a link.");
        printf("unistd/link.c inode: %ld\n", inode);
        printf("link.out inode: %ld\n", buf.st_ino);
    }

    // Remove link
    if (unlink("link.out")) {
        perror("unlink");
        exit(EXIT_FAILURE);
    }
    if (!stat("link.out", &buf) || errno != ENOENT) {
        perror("stat");
        exit(EXIT_FAILURE);
    }
}
