#include <errno.h>
#include <stdio.h>
#include <sys/stat.h>
#include <unistd.h>

int main() {
    printf("%ld\n", sizeof(struct stat));

    struct stat buf;

    if (stat("unistd/stat.c", &buf)) {
        perror("stat");
        return 1;
    }

    printf("st_mode: %u\n", buf.st_mode);
    printf("st_size: %lu\n", buf.st_size);
    printf("st_blksize: %lu\n", buf.st_blksize);
}
