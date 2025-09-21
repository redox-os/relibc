#include <assert.h>
#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

__attribute__((nonnull))
static int run_test(
    int dir,
    const char name[],
    char buf[PATH_MAX],
    const char expected[]
) {
    if (readlinkat(dir, name, buf, PATH_MAX) == -1) {
        perror("readlinkat");
        return -1;
    }
    if (strncmp(expected, buf, PATH_MAX) != 0) {
        fprintf(
            stderr,
            "readlinkat mismatch\n\tExpected: %s\n\tActual: %s\n",
            expected,
            buf
        );
        return -1;
    }

    return 0;
}

int main(void) {
    int status = EXIT_FAILURE;

    char template[] = "/tmp/rlatest.XXXXXX";
    if (!mkdtemp(template)) {
        perror("mkdtemp");
        goto bye;
    }
    int dir = open(template, O_DIRECTORY);
    if (dir == -1) {
        perror("open");
        goto rmtempdir;
    }

    // Set up file and link.
    size_t len = sizeof(template) - 1;
    const char file_name[] = "miku";
    char file_path[PATH_MAX] = {0};
    memcpy(file_path, template, len);
    file_path[len] = '/';
    memcpy(&file_path[len + 1], file_name, sizeof(file_name));
    int file = open(file_path, O_CREAT | O_WRONLY);
    if (file == -1) {
        perror("open");
        goto close_dir;
    }

    // Writing a sentinel message to the target file is useful in case
    // readlinkat reads the file instead of the link. If that happens,
    // it will self-evident instead of printing an empty string.
    // (n.b. this totally happened to me so I KNOW it's useful)
    const char msg[] = "readlinkat read the file instead of the link...oops";
    const ssize_t msg_len = sizeof(msg);
    assert(msg_len < PATH_MAX);
    if (write(file, msg, msg_len) < msg_len) {
        perror("write");
        goto rmfiles;
    }
    if (close(file) == -1) {
        perror("close");
        goto rmfiles;
    }
    file = -1;

    const char link_name[] = "link";
    char link_path[PATH_MAX] = {0};
    memcpy(link_path, template, len);
    link_path[len] = '/';
    memcpy(&link_path[len + 1], link_name, len);
    if (symlink(file_path, link_path) == -1) {
        perror("symlink");
        goto rmfiles;
    }

    // Relative path
    char buf[PATH_MAX] = {0};
    if (run_test(dir, link_name, buf, file_path) == -1) {
        fputs("Context: Basic test (relative path)\n", stderr);
        goto rmfiles;
    }

    // Absolute path
    memset(buf, 0, PATH_MAX);
    if (run_test(dir, link_path, buf, file_path) == -1) {
        fputs("Context: Absolute path\n", stderr);
        goto rmfiles;
    }

    // AT_FDCWD
    memset(buf, 0, PATH_MAX);
    char old_cwd[PATH_MAX] = {0};
    if (!getcwd(old_cwd, PATH_MAX)) {
        perror("getcwd");
        goto rmfiles;
    }
    if (chdir(template) == -1) {
        perror("chdir");
        goto rmfiles;
    }
    if (run_test(AT_FDCWD, link_name, buf, file_path) == -1) {
        fputs("Context: AT_FDCWD\n", stderr);
        goto rmfiles;
    }
    if (chdir(old_cwd) == -1) {
        perror("chdir");
        goto rmfiles;
    }

    // Not a dir
    memset(buf, 0, PATH_MAX);
    file = open(file_path, O_PATH);
    if (file == -1) {
        perror("open");
        goto rmfiles;
    }
    if (readlinkat(file, "", buf, PATH_MAX) != -1) {
        fputs("Context: Using a file for dirfd should fail\n", stderr);
        fprintf(stderr, "readlinkat wrote this into buf: %s\n", buf);
        goto close_file;
    }

    status = EXIT_SUCCESS;
close_file:
    close(file);
rmfiles:
    unlink(file_path);
    unlink(link_path);
close_dir:
    close(dir);
rmtempdir:
    rmdir(template);
bye:
    return status;
}
