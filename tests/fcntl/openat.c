#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <dirent.h>
#include <sys/stat.h>
#include <sys/types.h>

#include "test_helpers.h"

int main(void) {
    int m = mkdir("test_dir", 0777);
    ERROR_IF(mkdir, m, == -1);

    int fd_file = open("test_dir/test_file.txt", O_WRONLY | O_CREAT | O_TRUNC, 0644);
    ERROR_IF(open, fd_file, == -1);
    UNEXP_IF(open, fd_file, < 0);

    int c1 = close(fd_file);
    ERROR_IF(close, c1, == -1);

    DIR *dir_stream = opendir("test_dir");
    ERROR_IF(opendir, dir_stream, == NULL);

    int dfd = dirfd(dir_stream);
    ERROR_IF(dirfd, dfd, == -1);
    UNEXP_IF(dirfd, dfd, < 0);

    int fd_at = openat(dfd, "../test_dir/test_file.txt", O_RDONLY);
    ERROR_IF(openat, fd_at, == -1);
    UNEXP_IF(openat, fd_at, < 0);

    int c2 = close(fd_at);
    ERROR_IF(close, c2, == -1);

    int c3 = closedir(dir_stream);
    ERROR_IF(closedir, c3, == -1);

    int u = unlink("test_dir/test_file.txt");
    ERROR_IF(unlink, u, == -1);

    int r = rmdir("test_dir");
    ERROR_IF(rmdir, r, == -1);
}