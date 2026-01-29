#define _GNU_SOURCE

#include <assert.h>
#include <fcntl.h>
#include <limits.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <sys/stat.h>
#include <sys/wait.h>

__attribute__((nonnull))
int cloexec_test(char path[], int argc, char* argv[]) {
    assert(argc >= 1);

    int fd = open(path, O_RDONLY | O_CLOEXEC);
    if (fd == -1) {
        perror("open (O_CLOEXEC)");
        return -1;
    }

    pid_t child = fork();
    if (child == -1) {
        perror("fork");
        close(fd);
        return -1;
    } else if (child == 0) {
        // In child process where fd should be closed after execv.
        char fd_str[8] = {0};
        sprintf(fd_str, "%d", fd);

        char* new_argv[] = {
            argv[0],
            fd_str,
            NULL
        };

        if (execv(argv[0], new_argv) == -1) {
            perror("execv");
        }

        exit(EXIT_FAILURE);
    } else {
        // In parent where fd should remain open.
        int status = -1;
        if (waitpid(child, &status, 0) == -1) {
            perror("waitpid");
            return -1;
        }

        if (fcntl(fd, F_GETFD) < 0) {
            fputs(
                "File descriptor closed in parent after exec with O_CLOEXEC\n",
                stderr
            );
            kill(child, SIGKILL);
            waitpid(child, NULL, 0);

            return -1;
        }

        if (WIFEXITED(status) && WEXITSTATUS(status) == EXIT_SUCCESS) {
            return 0;
        } else {
            fputs(
                "Child process to test O_CLOEXEC failed\n",
                stderr
            );
            return -1;
        }
    }

    // Unreachable
    return -1;
}

int cloexec_validate(int argc, char* argv[]) {
    assert(argc == 2);

    char* end = NULL;
    int fd = (int) strtol(argv[1], &end, 0);

    if (*end == '\0') {
        if (fcntl(fd, F_GETFD) >= 0) {
            fputs(
                "File descriptor still open after exec with O_CLOEXEC\n",
                stderr
            );

            return EXIT_FAILURE;
        }
        return EXIT_SUCCESS;
    } else {
        fputs(
            "Invalid file descriptor number passed to cloxec_validate\n",
            stderr
        );
        return EXIT_FAILURE;
    }
}

int main(int argc, char* argv[]) {
    // The test runner inserts two fixed arg, so argc == 2 is fine
    if (argc == 2) {
        return cloexec_validate(argc, argv);
    } else if (argc == 0) {
        fprintf(
            stderr,
            "Invalid number of arguments: %d\n",
            argc
        );
        return EXIT_FAILURE;
    }

    int result = EXIT_FAILURE;

    char dir_template[] = "open_tests.XXXXXX";
    size_t dlen = sizeof(dir_template) - 1;
    if (!mkdtemp(dir_template)) {
        perror("mkdtemp");
        goto bye;
    }

    char file_path[PATH_MAX] = {0};
    const char file_name[] = "peanut";
    memcpy(file_path, dir_template, dlen);
    file_path[dlen] = '/';
    memcpy(&file_path[dlen + 1], file_name, sizeof(file_name));
    
    int fd = open(file_path, O_CREAT, 0644);
    if (fd == -1) {
        perror("open (creating file in temp dir)");
        goto clean_opened_dir;
    }
    if (close(fd) == -1) {
        perror("close (file)");
        goto clean_opened_dir;
    }

    // Check that close actually closed fd
    if (fcntl(fd, F_GETFD) >= 0) {
        fputs(
            "File descriptor was not closed by close\n",
            stderr
        );
        goto clean_opened_file;
    }

    // Opening a file with O_DIRECTORY should fail
    fd = open(file_path, O_DIRECTORY);
    if (fd != -1) {
        fputs(
            "File successfully opened with O_DIRECTORY\n",
            stderr
        );
        close(fd);
        goto clean_opened_file;
    }
    
    // O_CREAT | O_EXCL should fail on existing file
    fd = open(file_path, O_CREAT | O_EXCL, 0644);
    if (fd != -1) {
        fputs(
            "Existing file created with O_CREAT | O_EXCL\n",
            stderr
        );
        close(fd);
        goto clean_opened_file;
    }

    // O_PATH
    fd = open(file_path, O_PATH | O_WRONLY);
    // O_PATH should ignore O_WRONLY but NOT fail.
    if (fd == -1) {
        perror("open (O_PATH)");
        goto clean_opened_file;
    }

    // Writing this buf should fail.
    const char buf[] = "Valencia peanuts";
    if (write(fd, buf, sizeof(buf)) != -1) {
        fputs(
            "Writing to an O_PATH fd succeeded\n",
            stderr
        );
        close(fd);
        goto clean_opened_file;
    }

    // But fstat should succeed with O_PATH
    struct stat stat = {0};
    if (fstat(fd, &stat) == -1) {
        perror("fstat (O_PATH)");
        close(fd);
        goto clean_opened_file;
    }

    // O_NOATIME (disabled since Redox doesn't have it)
    // struct timespec atime_expected = stat.st_atim;

    close(fd);
    /* fd = open(file_path, O_NOATIME); */
    /* if (fd == -1) { */
    /*     perror("open (O_NOATIME)"); */
    /*     close(fd); */
    /*     goto clean_opened_file; */
    /* } */
    /**/
    /* if (fstat(fd, &stat) == -1) { */
    /*     perror("fstat (O_PATH)"); */
    /*     close(fd); */
    /*     goto clean_opened_file; */
    /* } */
    /**/
    /* struct timespec atime_actual = stat.st_atim; */
    /* if ( */
    /*     (atime_expected.tv_nsec != atime_actual.tv_nsec) */
    /*     || (atime_expected.tv_sec != atime_actual.tv_sec) */
    /* ) { */
    /*     fputs( */
    /*         "Access time changed with O_NOATIME\n", */
    /*         stderr */
    /*     ); */
    /*     close(fd); */
    /*     goto clean_opened_file; */
    /* } */
    /* close(fd); */

    // O_NOFOLLOW
    char sym_path[PATH_MAX] = {0};
    const char sym_name[] = "cashew";
    memcpy(sym_path, dir_template, dlen);
    sym_path[dlen] = '/';
    memcpy(&sym_path[dlen + 1], sym_name, sizeof(sym_name));

    if (symlink(file_path, sym_path) == -1) {
        perror("symlink");
        goto clean_opened_file;
    }
    fd = open(sym_path, O_NOFOLLOW);
    if (fd != -1) {
        fputs(
            "Symlink opened with O_NOFOLLOW\n",
            stderr
        );
        close(fd);
        goto clean_symlink;
    }

    // O_CLOEXEC
    cloexec_test(file_path, argc, argv);

    // Opening existing directory
    int dirfd = open(dir_template, O_RDONLY);
    if (dirfd == -1) {
        perror("open (existing dir)");
        goto clean_symlink;
    }
    close(dirfd);

    // Opening existing directory (read/write)
    dirfd = open(dir_template, O_RDWR);
    if (dirfd != -1) {
        fputs(
            "Opening a directory with O_RDWR incorrectly succeeded\n",
            stderr
        );
        goto clean_symlink;
    }
    close(dirfd);

    // Opening existing directory (write only; should fail)
    dirfd = -1;
    dirfd = open(dir_template, O_WRONLY);
    if (dirfd != -1) {
        fputs(
            "Opening a directory with O_WRONLY incorrectly succeeded\n",
            stderr
        );
        goto clean_symlink;
    }

    result = EXIT_SUCCESS;
clean_symlink:
    unlink(sym_path);
clean_opened_file:
    unlink(file_path);
clean_opened_dir:
    rmdir(dir_template);
bye:
    return result;
}
