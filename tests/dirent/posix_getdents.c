#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <dirent.h>
#include <errno.h>
#include <sys/stat.h>

#include "test_helpers.h"

#define BUFFER_SIZE 4096

void read_and_print_directory(int fd) {
    char buffer[BUFFER_SIZE];
    long nread;
    long bpos;
    struct dirent *d;

    for (;;) {
        nread = posix_getdents(fd, buffer, BUFFER_SIZE, 0);
        ERROR_IF(posix_getdents, nread, == -1);
        UNEXP_IF(posix_getdents, nread, < 0);

        if (nread == 0) {
            break;
        }

        for (bpos = 0; bpos < nread;) {
            d = (struct dirent *) (buffer + bpos);
            printf("  ino = %-10lu  name = %s\n", (unsigned long)d->d_ino, d->d_name);
            bpos += d->d_reclen;
        }
    }
}

int main(void) {
    int fd = open("example_dir/", O_RDONLY | O_DIRECTORY);
    ERROR_IF(open, fd, == -1);
    read_and_print_directory(fd);

    off_t seek_result = lseek(fd, 0, SEEK_SET);
    ERROR_IF(lseek, seek_result, == -1);
    read_and_print_directory(fd);

    close(fd);
}
