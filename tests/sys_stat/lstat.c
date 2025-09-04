#include <assert.h>
#include <fcntl.h>
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <sys/stat.h>

int main(void) {
    int status = EXIT_FAILURE;

    char tempdir_path[] = "/tmp/lstat_test.XXXXXX";
    if (!mkdtemp(tempdir_path)) {
        perror("mkdtemp");
        goto bye;
    }
    int tempdir = open(tempdir_path, O_DIRECTORY);
    if (tempdir == -1) {
        perror("open (tempdir)");
        goto rmtemp;
    }

    // Source
    const char src_name[] = "kerouac";
    char src_buf[PATH_MAX] = {0};
    strcpy(src_buf, tempdir_path);
    strcat(src_buf, "/");
    strcat(src_buf, src_name);
    // TODO: Use openat
    int source = open(src_buf, O_CREAT);
    if (source == -1) {
        perror("open");
        goto close_dir;
    }

    // Dest (link)
    const char dst_name[] = "on_the_road";
    char dst_buf[PATH_MAX] = {0};
    strcpy(dst_buf, tempdir_path);
    strcat(dst_buf, "/");
    strcat(dst_buf, dst_name);
    if (symlink(src_buf, dst_buf) == -1) {
        perror("symlink");
        goto rm_source;
    }

    struct stat src_stat = {0};
    if (fstatat(tempdir, dst_name, &src_stat, 0) == -1) {
        perror("fstatat");
        goto rm_dest;
    }
    // Quick consistency check
    uid_t uid = getuid();
    gid_t gid = getgid();
    if (src_stat.st_uid != uid) {
        fprintf(
            stderr,
            "uid:\n\tExpected %ld\n\tActual: %ld\n",
            (intmax_t) uid,
            (intmax_t) gid
        );
        goto rm_dest;
    }
    if (src_stat.st_nlink != 1) {
        fputs("nlink: Expected one hard link\n", stderr);
        goto rm_dest;
    }
    if ((src_stat.st_mode & S_IFMT) != S_IFREG) {
        fputs("mode: Expected a normal file\n", stderr);
        goto rm_dest;
    }

    struct stat dst_stat = {0};
    if (lstat(dst_buf, &dst_stat) == -1) {
        perror("lstat");
        goto rm_dest;
    }
    if (memcmp(&src_stat, &dst_stat, sizeof(struct stat)) == 0) {
        fputs("lstat incorrectly followed the symlink\n", stderr);
        goto rm_dest;
    }
    if ((dst_stat.st_mode & S_IFMT) != S_IFLNK) {
        fputs("lstat did not stat a link\n", stderr);
        goto rm_dest;
    }

    status = EXIT_SUCCESS;
rm_dest:
    unlink(dst_buf);
rm_source:
    unlink(src_buf);
close_dir:
    close(tempdir);
rmtemp:
    rmdir(tempdir_path);
bye:
    return status;
}
