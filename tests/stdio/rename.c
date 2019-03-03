#include <stdio.h>
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "test_helpers.h"

static char oldpath[] = "old-name.out";
static char newpath[] = "new-name.out";
static char str[] = "Hello, World!";

int main(void) {
    char buf[14] = { 0 };

    // Create old file
    int fd = creat(oldpath, 0777);
    ERROR_IF(creat, fd, == -1);
    UNEXP_IF(creat, fd, < 0);

    int written_bytes = write(fd, str, strlen(str));
    ERROR_IF(write, written_bytes, == -1);

    int c1 = close(fd);
    ERROR_IF(close, c1, == -1);
    UNEXP_IF(close, c1, != 0);

    // Rename old file to new file
    int rn_status = rename(oldpath, newpath);
    ERROR_IF(rename, rn_status, == -1);
    UNEXP_IF(rename, rn_status, != 0);

    // Read new file
    fd = open(newpath, O_RDONLY);
    ERROR_IF(open, fd, == -1);
    UNEXP_IF(open, fd, < 0);

    int read_bytes = read(fd, buf, strlen(str));
    ERROR_IF(read, read_bytes, == -1);
    UNEXP_IF(read, read_bytes, < 0);

    int c2 = close(fd);
    ERROR_IF(close, c2, == -1);
    UNEXP_IF(close, c2, != 0);

    // Remove new file
    int rm_status = remove(newpath);
    ERROR_IF(remove, rm_status, == -1);
    UNEXP_IF(remove, rm_status, != 0);

    // Compare file contents
    if (strcmp(str, buf) != 0) {
        puts("Comparison failed!");
        exit(EXIT_FAILURE);
    }
}
