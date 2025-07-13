#include <assert.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "test_helpers.h"

static char oldpath[] = "old-name.out";
static char newpath[] = "new-name.out";
static char str[] = "Hello, World!";

__attribute__((nonnull))
static char* join_paths(const char* dir, const char* name) {
    size_t dir_len = strlen(dir);
    // Length of directory, backslash, name, and NUL
    size_t full_len = dir_len + 2 + strlen(name);

    char* path = (char*) malloc(full_len);
    if (!path) {
        return NULL;
    }

    // SAFETY: Path is memset to 0 so it always ends in a NUL
    memset(path, 0, full_len);
    strncpy(path, dir, dir_len);
    path[dir_len] = '/';
    strncpy(&path[dir_len + 1], name, strlen(name));

    return path;
}

// Test that renaming a broken symbolic link works.
// Symlinks should not be resolved by rename.
// One of the problems that arises when links are resolved is that
// broken links can't be renamed.
//
// This test returns EXIT_FAILURE/EXIT_SUCCESS because it needs to clean
// up temporary files on failure. The test helpers call _exit.
int rename_broken_symlink(void) {
    char dir_template[] = "/tmp/sltest.XXXXXX";
    char* temp_dir = mkdtemp(dir_template);
    if (!temp_dir) {
        perror("mkdtemp");
        return EXIT_FAILURE;
    }

    // Broken link to be created
    const char link_name[] = "sym";
    const char* link_path = join_paths(temp_dir, link_name);
    if (!link_path) {
        fputs("Allocating string for symlink failed", stderr);

        rmdir(temp_dir);
        return EXIT_FAILURE;
    }

    // Non-existing target
    const char link_target[] = "target";
    const char* target_path = join_paths(temp_dir, link_target);
    if (!target_path) {
        fputs("Allocating string for link target path failed", stderr);

        free((void*) link_path);
        rmdir(temp_dir);

        return EXIT_FAILURE;
    }

    // New name of link (i.e. mv sym symrename)
    const char link_rename[] = "symrename";
    const char* rename_path = join_paths(temp_dir, link_rename);
    if (!rename_path) {
        fputs("Allocating string for renamed symlink failed", stderr);

        free((void*) target_path);
        free((void*) link_path);
        rmdir(temp_dir);
        
        return EXIT_FAILURE;
    }

    // Target most definitely does NOT exist.
    // This is a sanity check that shouldn't fail as test uses temp dirs.
    int target_fd = open(target_path, O_RDONLY);
    if (target_fd != -1) {
        fprintf(stderr,
                "Target exists when it shouldn't: %s\n",
                target_path
        );

        close(target_fd);
        free((void*) rename_path);
        free((void*) target_path);
        free((void*) link_path);
        rmdir(temp_dir);
                
        return EXIT_FAILURE;
    }

    // Create a broken symlink in a temp directory
    if (symlink(target_path, link_path) < 0) {
        perror("symlink");

        free((void*) rename_path);
        free((void*) target_path);
        free((void*) link_path);
        rmdir(temp_dir);

        return EXIT_FAILURE;
    }

    // Rename the link; this should work even if target doesn't exist
    if (rename(link_path, rename_path) < 0) {
        perror("rename");

        unlink(link_path);
        free((void*) rename_path);
        free((void*) target_path);
        free((void*) link_path);
        rmdir(temp_dir);

        return EXIT_FAILURE;
    }

    // TODO: Assert paths exist (needs openat)

    assert(unlink(rename_path) == 0);
    free((void*) rename_path);
    free((void*) target_path);
    free((void*) link_path);
    assert(rmdir(temp_dir) == 0);
    return EXIT_SUCCESS;
}

int main(void) {
    char buf[14] = { 0 };

    // Create old file
    int fd = creat(oldpath, 0777);
    ERROR_IF(creat, fd, == -1);
    UNEXP_IF(creat, fd, < 0);

    int written_bytes = write(fd, str, strlen(str));
    ERROR_IF(write, written_bytes, == -1);

    int c1 = close(fd);
    ERROR_IF(close, c1, == -1);
    UNEXP_IF(close, c1, != 0);

    // Rename old file to new file
    int rn_status = rename(oldpath, newpath);
    ERROR_IF(rename, rn_status, == -1);
    UNEXP_IF(rename, rn_status, != 0);

    // Read new file
    fd = open(newpath, O_RDONLY);
    ERROR_IF(open, fd, == -1);
    UNEXP_IF(open, fd, < 0);

    int read_bytes = read(fd, buf, strlen(str));
    ERROR_IF(read, read_bytes, == -1);
    UNEXP_IF(read, read_bytes, < 0);

    int c2 = close(fd);
    ERROR_IF(close, c2, == -1);
    UNEXP_IF(close, c2, != 0);

    // Remove new file
    int rm_status = remove(newpath);
    ERROR_IF(remove, rm_status, == -1);
    UNEXP_IF(remove, rm_status, != 0);

    // Compare file contents
    if (strcmp(str, buf) != 0) {
        puts("Comparison failed!");
        exit(EXIT_FAILURE);
    }

    // TEST DISABLED until relibc#212 fixed.
    // assert(rename_broken_symlink() == EXIT_SUCCESS);
}
