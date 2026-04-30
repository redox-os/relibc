#include <stdio.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <fcntl.h>
#include <unistd.h>
#include "test_helpers.h"

int main() {
    const char *test_file = "umask.txt";
    struct stat st;
    mode_t target_umask = 0022;
    mode_t creation_mode = 0666;
    mode_t expected_mode = 0644;

    umask(target_umask);

    int fd = open(test_file, O_WRONLY | O_CREAT | O_TRUNC, creation_mode);
    ERROR_IF(open, fd, == -1);
    close(fd);

    int stat_status = stat(test_file, &st);
    ERROR_IF(stat, stat_status, == -1);
    UNEXP_IF(stat, stat_status, != 0);

    mode_t actual_mode = st.st_mode & 0777;
    UNEXP_IF(mode_check, actual_mode, != expected_mode);

    int unlink_status = unlink(test_file);
    ERROR_IF(unlink, unlink_status, == -1);
    UNEXP_IF(unlink, unlink_status, != 0);

    return 0;
}