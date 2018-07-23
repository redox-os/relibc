#include <errno.h>
#include <stdio.h>
#include <sys/stat.h>
#include <unistd.h>

int main(int argc, char** argv) {
    printf("%ld\n", sizeof(struct stat));

    struct stat buf;

    // Stat for the inode
    if (stat("unistd/link.c", &buf)) {
        perror("stat");
        return 1;
    }
    unsigned long inode = buf.st_ino;
    printf("%ld\n", inode);

    // Create the link
    if (link("unistd/link.c", "link.out")) {
        perror("link");
        return 1;
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
        return 1;
    }
    if (!stat("link.out", &buf) || errno != ENOENT) {
        perror("stat");
        return 1;
    }
}
