#define _GNU_SOURCE

#include <fcntl.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int temp_file(
    const char dir[],
    size_t dlen,
    char buf[PATH_MAX],
    const char name[],
    size_t nlen,
    bool create
) {
    memcpy(buf, dir, dlen);
    buf[dlen] = '/';
    memcpy(&buf[dlen + 1], name, nlen);

    if (create) {
        int fd = open(buf, O_CREAT);
        if (fd == -1) {
            perror("open (file A)");
            close(fd);
            return -1;
        }
        close(fd);
    }

    return 0;
}

int main(void) {
    int result = EXIT_FAILURE;

    char dir_template[] = "/tmp/mvattest.XXXXXX";
    size_t dlen = sizeof(dir_template) - 1;
    if (!mkdtemp(dir_template)) {
        perror("mkdtemp");
        goto bye;
    }
    int dirfd = open(dir_template, O_DIRECTORY | O_PATH);
    if (dirfd == -1) {
        perror("open (temp directory)");
        goto bye;
    }

    char dir_template2[] = "/tmp/mvattest2.XXXXXX";
    size_t dlen2 = sizeof(dir_template2) - 1;
    if (!mkdtemp(dir_template2)) {
        perror("mkdtemp");
        goto close_dir1;
    }
    int dirfd2 = open(dir_template2, O_DIRECTORY | O_PATH);
    if (dirfd2 == -1) {
        perror("open (temp directory)");
        goto close_dir1;
    }

    // File A
    const char file_a[] = "power";
    char file_a_path[PATH_MAX] = {0};
    if (temp_file(dir_template, dlen, file_a_path, file_a, sizeof(file_a), true) == -1) {
        goto close_dir2;
    }
    
    // File B
    const char file_b[] = "nyanko";
    char file_b_path[PATH_MAX] = {0};
    if (
        temp_file(
            dir_template,
            dlen,
            file_b_path,
            file_b,
            sizeof(file_b),
            false
        ) == -1
    ) {
        goto remove_ab;
    }

    // File A but in directory 2
    char file_a_dir2[PATH_MAX] = {0};
    if (
        temp_file(
            dir_template2,
            dlen2,
            file_a_dir2,
            file_a,
            sizeof(file_a),
            false
        ) == -1
    ) {
        goto remove_all;
    }

    // Move file A to file B normally (same dir)
    if (renameat(dirfd, file_a, dirfd, file_b) == -1) {
        perror("renameat (A -> B, basic)");
        goto remove_all;
    }
    if (access(file_b_path, F_OK) == -1) {
        perror("access (file B, basic)");
        goto remove_all;
    }

    // Move file B to A (absolute path; same dir)
    if (renameat(dirfd, file_b_path, dirfd, file_a) == -1) {
        perror("renameat (B -> A, absolute path)");
        goto remove_all;
    }
    if (access(file_a_path, F_OK) == -1) {
        perror("access (file A, absolute path)");
        goto remove_all;
    }

    // Move A to B (both absolute)
    if (renameat(dirfd, file_a_path, dirfd, file_b_path) == -1) {
        perror("renameat (A -> B, both absolute)");
        goto remove_all;
    }
    if (access(file_b_path, F_OK) == -1) {
        perror("access (file B, both absolute)");
        goto remove_all;
    }

    // Move B to B
    if (renameat(dirfd, file_b, dirfd, file_b) == -1) {
        perror("renameat (B -> B)");
        goto remove_all;
    }
    if (access(file_b_path, F_OK) == -1) {
        perror("access (file B)");
        goto remove_all;
    }

    // Move file B to A (AT_FDCWD)
    char cwd[PATH_MAX] = {0};
    if (!getcwd(cwd, PATH_MAX)) {
        perror("getcwd");
        goto remove_all;
    }
    if (chdir(dir_template) == -1) {
        perror("chdir");
        goto remove_all;
    }

    if (renameat(AT_FDCWD, file_b, dirfd, file_a) == -1) {
        perror("renameat (B -> A, AT_FDCWD)");
        goto remove_all;
    }
    if (access(file_a_path, F_OK) == -1) {
        perror("access (file A, AT_FDCWD)");
        goto remove_all;
    }

    // Reset, though it doesn't really matter.
    if (chdir(cwd) == -1) {
        perror("chdir");
        goto remove_all;
    }

    // Move to different directory
    if (renameat(dirfd, file_a, dirfd2, file_a) == -1) {
        perror("renameat (A -> B, different dirs)");
        goto remove_all;
    }
    if (access(file_a_dir2, F_OK) == -1) {
        perror("access (file A in dir2)");
        goto remove_all;
    }

    // Move non-existing file
    if (renameat(dirfd, "aki", dirfd, "denji") == 0) {
        // Wut?
        fputs("renameat succeeded at moving a non-existing file\n", stderr);
        goto remove_all;
    }

    // RENAME_NOREPLACE
    // Create file B in dir 1 first because we moved it earlier.
    int fd = open(file_b_path, O_CREAT);
    if (fd == -1) {
        perror("open (file B in dir 1)");
        close(fd);
        goto remove_all;
    }
    close(fd);

    if (renameat2(dirfd, file_b, dirfd2, file_a, RENAME_NOREPLACE) == 0) {
        fputs("renameat2 (B -> A, noreplace) succeeded\n", stderr);
        goto remove_all;
    }
    if (access(file_a_dir2, F_OK) == -1) {
        fputs("RENAME_NOREPLACE ate file A in dir 2\n", stderr);
        goto remove_all;
    }
    if (access(file_b_path, F_OK) == -1) {
        fputs("RENAME_NOREPLACE ate file B in dir 1\n", stderr);
        goto remove_all;
    }

    // TODO: RENAME_EXCHANGE (Needs frename support in Redox)
    // Notes for later:
    // * Write a message to both files
    // * Swap
    // * Check if the files swapped correctly by strcmp messages

    result = EXIT_SUCCESS;
remove_all:
    remove(file_a_dir2);
remove_ab:
    remove(file_a_path);
    remove(file_b_path);
    rmdir(dir_template);
    rmdir(dir_template2);
close_dir2:
    close(dirfd2);
close_dir1:
    close(dirfd);
bye:
    return result;
}
