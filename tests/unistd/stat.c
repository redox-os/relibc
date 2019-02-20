#include <errno.h>
#include <stdio.h>
#include <sys/stat.h>
#include <unistd.h>

int main(void) {
    printf("%ld\n", sizeof(struct stat));

    struct stat buf;

    if (stat("unistd/stat.c", &buf)) {
        perror("stat");
        return 1;
    }

    printf("st_size: %lu\n", buf.st_size);
    printf("st_blksize: %lu\n", buf.st_blksize);
    printf("st_dev: %lu\n", buf.st_dev);
    printf("st_ino: %lu\n", buf.st_ino);
    printf("st_mode: %o\n", buf.st_mode);
    printf("st_nlink: %lu\n", buf.st_nlink);
    printf("st_uid: %u\n", buf.st_uid);
    printf("st_gid: %u\n", buf.st_gid);
}
