#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <sys/socket.h>
#include <sys/stat.h>

__attribute__((nonnull))
int run_test(
    int filefd,
    const char error_msg[],
    off_t offset,
    off_t length,
    int dirfd,
    const char file_name[],
    off_t expected_size
) {
    // posix_fallocate does not set errno.
    int result = posix_fallocate(filefd, offset, length);
    if (result != 0) {
        const char* error = strerror(result);
        fprintf(stderr, "%s: %s\n", error_msg, error);
        return -1;
    }

    struct stat stat = {0};
    if (fstatat(dirfd, file_name, &stat, 0) == -1) {
        perror("fstatat");
        return -1;
    }

    if (stat.st_size != expected_size) {
        fprintf(
            stderr,
            "Expected: %zd\nActual: %zd\n",
            expected_size,
            stat.st_size
        );
        return -1;
    }

    return 0;
}

int main(void) {
    int result = EXIT_FAILURE;

    char dir_template[] = "/tmp/pfalloctest.XXXXXX";
    size_t dlen = sizeof(dir_template) - 1;
    if (!mkdtemp(dir_template)) {
        perror("mkdtemp");
        goto bye;
    }
    int dirfd = open(dir_template, O_DIRECTORY);
    if (dirfd == -1) {
        perror("open");
        goto rmtemp;
    }

    // Failure conditions.
    // These are mostly a gut check for whatever backing syscall is used
    // in relibc. If it ever changes, we have to make sure the correct
    // errors are returned. If they're not, we have to add checks to our
    // relibc implementation. Linux's backing syscall (SYS_fallocate)
    // already does these checks.
    //
    // Directory:
    if (posix_fallocate(dirfd, 0, 1000) == 0) {
        fputs("posix_fallocate succeeded on a directory\n", stderr);
        goto rmtemp;
    }

    // Socket:
    int sock = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sock == -1) {
        perror("socket");
        goto rmtemp;
    }
    if (posix_fallocate(sock, 0, 1000) == 0) {
        fputs("posix_fallocate succeeded on a socket\n", stderr);
        close(sock);
        goto rmtemp;
    }
    close(sock);

    // Pipe:
    int pipefd[2] = {0};
    if (pipe(pipefd) == -1) {
        perror("pipe");
        goto rmtemp;
    }
    if (posix_fallocate(pipefd[0], 0, 1000) == 0) {
        fputs("posix_fallocate succeeded on a pipe\n", stderr);
        close(pipefd[0]);
        close(pipefd[1]);
        goto rmtemp;
    }
    close(pipefd[0]);
    close(pipefd[1]);

    // Success conditions.
    const char name[] = "commander_keen";
    char path[PATH_MAX] = {0};
    memcpy(path, dir_template, dlen);
    path[dlen] = '/';
    memcpy(&path[dlen + 1], name, sizeof(name));

    int file = open(path, O_CREAT | O_RDWR);
    if (file == -1) {
        perror("open");
        goto rmtemp;
    }

    // Expand an empty file.
    if (
        run_test(
            file,
            "posix_fallocate failed expanding an empty file",
            0,
            1000,
            dirfd,
            name,
            1000
        ) == -1
    ) {
        fputs("FAILED: Expanding empty file test\n", stderr);
        goto rmtempfile;
    }

    // Don't shrink a file.
    if (
        run_test(
            file,
            "posix_fallocate failed on already allocated file",
            0,
            1,
            dirfd,
            name,
            1000
        ) == -1
    ) {
        fputs("FAILED: posix_fallocate should not shrink files\n", stderr);
        goto rmtempfile;
    }

    // Don't overwrite allocated byte ranges.
    if (ftruncate(file, 0) == -1) {
        perror("ftruncate");
        goto rmtempfile;
    }
    const char msg[] = "If you're reading this you must play Commander Keen";
    if (write(file, msg, sizeof(msg)) != sizeof(msg)) {
        perror("write");
        goto rmtempfile;
    }

    if (
        run_test(
            file,
            "posix_fallocate failed on an allocated range",
            0,
            sizeof(msg),
            dirfd,
            name,
            sizeof(msg)
        ) == -1
    ) {
        fputs(
            "FAILED: posix_fallocate don't overwrite allocated ranges test\n",
            stderr
        );
        goto rmtempfile;
    }
    // And now check that the range wasn't overwritten.
    char buf[sizeof(msg)] = {0};
    if (lseek(file, 0, SEEK_SET) == -1) {
        perror("lseek");
        goto rmtempfile;
    }
    if (read(file, buf, sizeof(msg)) != sizeof(msg)) {
        perror("read");
        goto rmtempfile;
    }
    if (strncmp(msg, buf, sizeof(msg)) != 0) {
        fputs("FAILED: posix_fallocate overwrote/destroyed a file\n", stderr);
        goto rmtempfile;
    }

    // Offset + length that goes beyond file end expands file.
    if (
        run_test(
            file,
            "posix_fallocate failed on file expansion test",
            sizeof(msg),
            1000,
            dirfd,
            name,
            sizeof(msg) + 1000
        ) == -1
    ) {
        fputs("FAILED: posix_fallocate file expansion test\n", stderr);
        goto rmtempfile;
    }

    result = EXIT_SUCCESS;
rmtempfile:
    close(file);
    unlink(path);
rmtemp:
    rmdir(dir_template);
bye:
    return result;
}

