#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <sys/stat.h>

__attribute__((nonnull))
static bool check_dot(struct dirent* dirent) {
    const char* dots[2] = {
        ".",
        ".."
    };

    for (size_t i = 0; i < 2; i++) {
        if (strcmp(dots[i], dirent->d_name) == 0) {
            return true;
        }        
    }

    return false;
}

int main(void) {
    int status = EXIT_FAILURE;

    char template[] = "/tmp/fdotest.XXXXXX";
    if (!mkdtemp(template)) {
        perror("mkdtemp");
        goto bye;
    }

    const char* movies[] = {
        "big_lebowski",
        "blade_runner",
        "grand_budapest_hotel",
        "taxi_driver"
    };
    const size_t movies_len = sizeof(movies)/sizeof(char*);
    char paths[sizeof(movies)/sizeof(char*)][PATH_MAX] = {0};
    for (size_t i = 0; i < movies_len; ++i) {
        // Concat the path
        const size_t len = sizeof(template) - 1;
        char buf[PATH_MAX] = {0};
        memcpy(buf, template, len);
        buf[len] = '/';

        memcpy(&buf[len + 1], movies[i], strlen(movies[i]));
        memcpy(paths[i], buf, PATH_MAX);

        // And now create the file
        int fd = open(buf, O_CREAT);
        if (fd == -1) {
            perror("open");
            goto rmfiles;
        }
        close(fd);
    }

    // FIXME: Redox requires read perms for the dir while Linux/BSD don't.
    int dir = open(template, O_DIRECTORY | O_RDONLY);
    if (dir == -1) {
        perror("open");
        goto rmfiles;
    }

    DIR* iter = fdopendir(dir);
    if (!iter) {
        perror("fdopendir");
        goto closedirfd;
    }

    for (size_t i = 0; i < movies_len; ++i) {
        errno = 0;

        struct dirent* dirent = readdir(iter);
        if (!dirent) {
            if (errno) {
                perror("readdir");
            }
            fprintf(
                stderr,
                "Expected entry #%lu but directory stream is complete\n",
                i
            );

            goto closediriter;
        }

        // Skip . and ..
        if (check_dot(dirent)) {
            continue;
        }

        // Check that the entry matches one of the names.
        // readdir's order is indeterministic and looping over the names
        // is simpler than qsort for a test.
        for (size_t j = 0; j < movies_len; ++j) {
            if (strcmp(movies[j], dirent->d_name) == 0) {
                goto continue_outer;
            }
        }

        fprintf(
            stderr,
            "Unexpected entry: %s\n",
            dirent->d_name
        );
        goto closediriter;

    continue_outer:
        continue;
    }

    // fdclosedir returns ownership of the original fd.
    int returned_fd = fdclosedir(iter);
    // Internally, both closedir and fdclosedir consume the boxed DIR.
    iter = NULL;
    if (returned_fd != dir) {
        fputs("fdclosedir returned the wrong descriptor\n", stderr);
        goto closedirfd;
    }

    // Check that the file descriptor is still valid.
    struct stat stat = {0};
    if (fstatat(returned_fd, "", &stat, AT_EMPTY_PATH) == -1) {
        perror("fstatat");
        fputs("fdclosedir shouldn't have closed the fd\n", stderr);
        goto closedirfd;
    }

    status = EXIT_SUCCESS;
closediriter:
    if (iter) {
        closedir(iter);
    }
closedirfd:
    close(dir);
rmfiles:
    for (size_t i = 0; i < movies_len; ++i) {
        if (strnlen(paths[i], PATH_MAX) > 4) {
            unlink(paths[i]);
        }
    }
    rmdir(template);
bye:
    return status;
}
