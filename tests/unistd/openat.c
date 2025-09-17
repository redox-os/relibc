#include <fcntl.h>

#include "test_helpers.h"

#define TEST_FILENAME "testfile.txt"
#define TEST_CONTENT "hello openat\n"

int create_temp_folder(char *template) {
    char *dir = mkdtemp(template);
    if (!dir) {
        perror("mkdtemp");
        return -1;
    }
    int dirfd = open(dir, O_DIRECTORY);
    ERROR_IF(open, dirfd, == -1);
    return dirfd;
}

void test_create_and_write(int dirfd) {
    int fd = openat(dirfd, TEST_FILENAME, O_CREAT | O_WRONLY, 0644);
    ERROR_IF(openat, fd, == -1);

    ssize_t written = write(fd, TEST_CONTENT, strlen(TEST_CONTENT));
    ERROR_IF(write, written, != (ssize_t)strlen(TEST_CONTENT));

    int res = close(fd);
    ERROR_IF(close, res, == -1);
}

void test_open_and_read(int dirfd) {
    char buf[128] = {0};
    int fd = openat(dirfd, TEST_FILENAME, O_RDONLY);
    ERROR_IF(openat, fd, == -1);

    ssize_t read_bytes = read(fd, buf, sizeof(buf) - 1);
    ERROR_IF(read, read_bytes, == -1);

    if (strncmp(buf, TEST_CONTENT, strlen(TEST_CONTENT)) != 0) {
        perror("Content mismatch!\n");
        exit(EXIT_FAILURE);
    }

    int res = close(fd);
    ERROR_IF(close, res, == -1);
}

void test_open_nonexistent(int dirfd) {
    int fd = openat(dirfd, "doesnotexist.txt", O_RDONLY);
    ERROR_IF(openat, fd, != -1);
    CHECK_AND_PRINT_ERRNO(ENOENT);
}

void test_invalid_flags(int dirfd) {
    int fd = openat(dirfd, TEST_FILENAME, -1);
    ERROR_IF(openat, fd, != -1);
}

void test_invalid_dirfd() {
    int fd = openat(-1, TEST_FILENAME, O_RDONLY);
    ERROR_IF(openat, fd, != -1);
    CHECK_AND_PRINT_ERRNO(EBADF);
}

int main(void) {
    char template[] = "/tmp/openat_test.XXXXXX";
    int dirfd = create_temp_folder(template);
    ERROR_IF(create_temp_folder, dirfd, == -1);

    test_create_and_write(dirfd);
    test_open_and_read(dirfd);
    test_open_nonexistent(dirfd);
    test_invalid_flags(dirfd);
    test_invalid_dirfd();

    // Cleanup
    unlink(TEST_FILENAME);
    close(dirfd);
    rmdir(template);

    return 0;
}