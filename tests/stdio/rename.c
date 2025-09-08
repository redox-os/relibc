#include <assert.h>
#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "test_helpers.h"

static char oldpath[] = "old-name.out";
static char newpath[] = "new-name.out";
static char str[] = "Hello, World!";

// Test that renaming a broken symbolic link works.
// Symlinks should not be resolved by rename.
// One of the problems that arises when links are resolved is that
// broken links can't be renamed.
//
// This test returns EXIT_FAILURE/EXIT_SUCCESS because it needs to clean
// up temporary files on failure. The test helpers call _exit.
int rename_broken_symlink(void) {
    int result = EXIT_FAILURE;

    char dir_template[] = "/tmp/sltest.XXXXXX";
    size_t dlen = sizeof(dir_template) - 1;
    char* temp_dir = mkdtemp(dir_template);
    if (!temp_dir) {
        perror("mkdtemp");
        return EXIT_FAILURE;
    }

    // TODO: Almost all of the code below can be vastly simplified
    // with openat/symlinkat later.

    // Broken link to be created
    const char link_name[] = "sym";
    char link_path[PATH_MAX] = {0};
    memcpy(link_path, temp_dir, dlen);
    link_path[dlen] = '/';
    memcpy(&link_path[dlen + 1], link_name, sizeof(link_name));

    // Non-existing target
    const char link_target[] = "target";
    char target_path[PATH_MAX] = {0};
    memcpy(target_path, temp_dir, dlen);
    target_path[dlen] = '/';
    memcpy(&target_path[dlen + 1], link_target, sizeof(link_target));

    // New name of link (i.e. mv sym symrename)
    const char link_rename[] = "symrename";
    char rename_path[PATH_MAX] = {0};
    memcpy(rename_path, temp_dir, dlen);
    rename_path[dlen] = '/';
    memcpy(&rename_path[dlen + 1], link_rename, sizeof(link_rename));

    // Target most definitely does NOT exist.
    // This is a sanity check that shouldn't fail as test uses temp dirs.
    int target_fd = open(target_path, O_RDONLY);
    if (target_fd != -1) {
        fprintf(stderr,
                "Target exists when it shouldn't: %s\n",
                target_path
        );
        // Skip clean up on the very exceptional case that the
        // randomized dir and file exists.
        goto skip_cleanup;
    }

    // Create a broken symlink in a temp directory
    if (symlink(target_path, link_path) < 0) {
        perror("symlink");
        goto cleanup;
    }

    // Rename the link; this should work even if target doesn't exist
    if (rename(link_path, rename_path) < 0) {
        perror("rename");
        goto cleanup;
    }

    // TODO: Assert paths exist (needs openat)

    result = EXIT_SUCCESS;
cleanup:
    unlink(link_path);
    unlink(rename_path);
    rmdir(temp_dir);
skip_cleanup:
    return result;
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

    int broken_symlink_res = rename_broken_symlink();
    assert(broken_symlink_res == EXIT_SUCCESS);
}
