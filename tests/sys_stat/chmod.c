#define _GNU_SOURCE // Linux to run locally

#include <fcntl.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <sys/stat.h>

/* #if defined(__linux__) && !defined(__redox__) */
/* #include <linux/version.h> */
/* #else */
/* #define KERNEL_VERSION(x,y,z) (0) */
/* #endif */

__attribute__((nonnull(2)))
static bool check_mode(
    int dir,
    const char path[],
    int flags,
    mode_t expected
) {
    struct stat stat = {0};
    if (fstatat(dir, path, &stat, flags) == -1) {
        perror("fstatat");
        return false;
    }

    if ((stat.st_mode & 0777) != expected) {
        fprintf(
            stderr,
            "Mode mismatch\n\tExpected: %o\n\tActual: %o\n",
            expected,
            stat.st_mode & 0777
        );
        return false;
    }

    return true;
}

int main(void) {
    int status = EXIT_FAILURE;

    char template[] = "/tmp/chmtest.XXXXXX";
    if (!mkdtemp(template)) {
        perror("mkdtemp");
        goto bye;
    }
    size_t len = sizeof(template) - 1;

    // Create a file and a link to chmod.
    const char file_name[] = "king_crimson";
    char file_path[PATH_MAX] = {0};
    memcpy(file_path, template, len);
    file_path[len] = '/';
    memcpy(&file_path[len + 1], file_name, sizeof(file_name));
    
    int filefd = open(file_path, O_CREAT);
    if (filefd == -1) {
        perror("open (file, O_CREAT)");
        goto rmtempdir;
    }

    const char link_name[] = "red";
    char link_path[PATH_MAX] = {0};
    memcpy(link_path, template, len);
    link_path[len] = '/';
    memcpy(&link_path[len + 1], link_name, sizeof(link_name));

    if (symlink(file_path, link_path) == -1) {
        perror("symlink");
        goto rmfiles;
    }
    #ifdef __redox__
    int linkfd = open(link_path, O_PATH | O_NOFOLLOW | O_SYMLINK);
    if (linkfd == -1) {
        perror("open (link, O_NOFOLLOW)");
        goto rmfiles;
    }
    #endif

    int dir = open(template, O_DIRECTORY);
    if (dir == -1) {
        perror("open (temp dir)");
        goto closelink;
    }

    // chmod

    // File
    if (chmod(file_path, 0666) == -1) {
        perror("chmod file 0666");
        goto closetemp;
    }
    if (!check_mode(dir, file_name, 0, 0666)) {
        fprintf(stderr, "Context: chmod %s\n", file_name);
        goto closetemp;
    }

    // Link (deferenced)
    if (chmod(link_path, 0777) == -1) {
        perror("chmod link 0777");
        goto closetemp;
    }
    if (!check_mode(dir, file_name, 0, 0777)) {
        fprintf(stderr, "Context: chmod %s\n", link_name);
        goto closetemp;
    }

    // Dir
    if (chmod(template, 0766) == -1) {
        perror("chmod directory 0766");
        goto closetemp;
    }
    if (!check_mode(dir, "", AT_EMPTY_PATH, 0766)) {
        fprintf(stderr, "Context: chmod %s\n", template);
        goto closetemp;
    }

    // fchmod

    // File
    if (fchmod(filefd, 0644) == -1) {
        perror("fchmod file 0644");
        goto closetemp;
    }
    if (!check_mode(dir, file_name, 0, 0644)) {
        fprintf(stderr, "Context: fchmod %s\n", file_name);
        goto closetemp;
    }

    // Link (not followed)
    #ifdef __redox__
    if (fchmod(linkfd, 0666) == -1) {
        perror("fchmod link 0666");
        goto closetemp;
    }
    if (!check_mode(dir, link_name, AT_SYMLINK_NOFOLLOW, 0666)) {
        fprintf(stderr, "Context: fchmod %s\n", link_name);
        goto closetemp;
    }
    #endif

    // Directory
    if (fchmod(dir, 0777) == -1) {
        perror("fchmod directory 0777");
        goto closetemp;
    }
    if (!check_mode(dir, "", AT_EMPTY_PATH, 0777)) {
        fprintf(stderr, "Context: fchmod %s\n", template);
        goto closetemp;
    }

    // fchmodat

    // File (relative)
    if (fchmodat(dir, file_name, 0666, 0) == -1) {
        perror("fchmodat directory 0666");
        goto closetemp;
    }
    if (!check_mode(dir, file_name, 0, 0666)) {
        fprintf(stderr, "Context: fchmodat %s\n", file_name);
        goto closetemp;
    }

    // File (absolute)
    if (fchmodat(dir, file_path, 0777, 0) == -1) {
        perror("fchmodat file 0777");
        goto closetemp;
    }
    if (!check_mode(dir, file_name, 0, 0777)) {
        fprintf(stderr, "Context: fchmodat %s\n", file_path);
    }

    // Link (followed)
    if (fchmodat(dir, link_name, 0666, 0) == -1) {
        perror("fchmodat link 0666");
        goto closetemp;
    }
    if (!check_mode(dir, file_name, 0, 0666)) {
        fprintf(stderr, "Context: fchmodat %s\n", link_name);
        goto closetemp;
    }

    // Link (not followed)
    #ifdef __redox__
    if (fchmodat(dir, link_name, 0777, AT_SYMLINK_NOFOLLOW) == -1) {
        perror("fchmodat link 0777");
        goto closetemp;
    }
    if (!check_mode(dir, link_name, AT_SYMLINK_NOFOLLOW, 0777)) {
        fprintf(stderr, "Context: fchmodat %s\n", link_name);
        goto closetemp;
    }
    #endif

    // Directory
    // AT_EMPTY_PATH support is relatively new so this fails in CI.
    #if defined(__redox__) // || LINUX_VERSION_CODE >= KERNEL_VERSION(6,6,0)
    if (fchmodat(dir, "", 0700, AT_EMPTY_PATH) == -1) {
        perror("fchmodat directory 0700");
        goto closetemp;
    }
    if (!check_mode(dir, "", AT_EMPTY_PATH, 0700)) {
        fprintf(stderr, "Context: fchmodat %s\n", template);
        goto closetemp;
    }

    // Directory (cwd)
    char old_cwd[PATH_MAX] = {0};
    if (!getcwd(old_cwd, PATH_MAX)) {
        perror("getcwd");
        goto closetemp;
    }
    if (chdir(template) == -1) {
        perror("chdir (temp dir)");
        goto closetemp;
    }

    if (fchmodat(AT_FDCWD, "", 0777, AT_EMPTY_PATH) == -1) {
        perror("fchmodat cwd 0777");
        goto closetemp;
    }
    if (!check_mode(AT_FDCWD, "", AT_EMPTY_PATH, 0777)) {
        fprintf(stderr, "Context: fchmodat %s\n", template);
        goto closetemp;
    }

    if (chdir(old_cwd) == -1) {
        perror("chdir (old cwd)");
        goto closetemp;
    }
    #endif

    status = EXIT_SUCCESS;
closetemp:
    close(dir);
closelink:
    #ifdef __redox__
    close(linkfd);
    #endif
rmfiles:
    close(filefd);
    unlink(file_path);
    unlink(link_path);
rmtempdir:
    rmdir(template);
bye:
    return status;
}
