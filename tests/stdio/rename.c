#include <stdio.h>
#include <fcntl.h>
#include <string.h>
#include <unistd.h>

static char oldpath[] = "old-name.out";
static char newpath[] = "new-name.out";
static char str[] = "Hello, World!";
int str_len = 13;

int main() {
    char buf[14];
    buf[13] = 0x00;
    int fd = creat(oldpath, 0777);
    write(fd, str, str_len);
    close(fd);
    rename(oldpath, newpath);
    fd = open(newpath, O_RDONLY);
    read(fd, buf, str_len);
    close(fd);
    remove(newpath);
    if (strcmp(str, buf) == 0) {
        return 0;
    } else {
        return -1;
    }
}
