#include <fcntl.h>
#include <stdio.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/time.h>
#include <unistd.h>
#include "test_helpers.h"

int main() {
    const char *test_file = "utimes.txt";
    struct stat st;
    struct timeval times[2];

    int fd = open(test_file, O_WRONLY | O_CREAT | O_TRUNC, 0666);
    ERROR_IF(open, fd, == -1);
    close(fd);

    times[0].tv_sec = 100;
    times[0].tv_usec = 0;
    times[1].tv_sec = 200;
    times[1].tv_usec = 0;

    int utime_status = utimes(test_file, times); 
    ERROR_IF(utimes, utime_status, == -1);

    int stat_status = stat(test_file, &st);
    ERROR_IF(stat, stat_status, == -1);

    UNEXP_IF(utimes_atime, st.st_atime, != times[0].tv_sec);
    UNEXP_IF(utimes_mtime, st.st_mtime, != times[1].tv_sec);

    int unlink_status = unlink(test_file);
    ERROR_IF(unlink, unlink_status, == -1);

    return 0;
}