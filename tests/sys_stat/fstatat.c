#include <assert.h>
#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <sys/stat.h>

// Create file `dir/name` and fill it with `contents`.
__attribute__((nonnull))
static const char* mktestfile(
        const char dir[],
        const char name[],
        const char contents[]
) {
    size_t dir_len = strlen(dir);
    size_t name_len = strlen(name);

    // dir + / name + \0
    char* path = malloc(dir_len + name_len + 2);
    if (!path) {
        perror("malloc");
        return NULL;
    }
    // memcpy is faster/recommended but I'm lazy.
    strcpy(path, dir);
    strcat(path, "/");
    strcat(path, name);

    int fd = creat(path, 0700);
    if (fd < 0) {
        perror("creat");
        free(path);
        return NULL;
    }

    size_t contents_len = strlen(contents);
    // Assumes that the handful of bytes of contents are fully written.
    if (write(fd, contents, contents_len) < 0) {
        perror("write");
        free(path);
        close(fd);
        return NULL;
    }

    close(fd);
    return path;
}

// Create a link, `dir/name`, to `target` and return its path.
__attribute__((nonnull))
static const char* mktestlink(
    const char dir[],
    const char name[],
    const char target[]
) {
    char* path = malloc(strlen(dir) + strlen(name) + 2);
    if (!path) {
        perror("malloc");
        return NULL;
    }

    strcpy(path, dir);
    strcat(path, "/");
    strcat(path, name);

    if (link(target, path) == -1) {
        perror("link");
        free(path);
        return NULL;
    }

    return path;
}

__attribute__((nonnull))
static int run_test(
    int fd,
    int flags,
    const char name[],
    const char path[],
    size_t expected
) {
    struct stat stat = {0};

    if (fstatat(fd, name, &stat, flags) == -1) {
        perror("fstatat");
        return -1;
    }
    if ((size_t) stat.st_size != expected) {
        fprintf(
            stderr,
            "fstatat invalid stats\n\tFile: %s\n\tExpected: %zu Actual: %ld\n",
            path,
            expected,
            stat.st_size
        );
        return -1;
    }

    return 0;
}

int main(void) {
    int status = EXIT_FAILURE;

    char dir_template_A[] = "/tmp/fsatest.XXXXXXX";
    char* dir_A = mkdtemp(dir_template_A);
    if (!dir_A) {
        perror("mkdtemp (dir A)");
        goto bye;
    }

    char dir_template_B[] = "/tmp/fsatest.XXXXXXX";
    char* dir_B = mkdtemp(dir_template_B);
    if (!dir_B) {
        perror("mkdtemp (dir B)");
        goto clean_dir_a;
    }

    // File in directory A
    const char name[] = "file";
    const char cont_A[] = "Flying Dutchman";
    const char* file_A = mktestfile(dir_A, name, cont_A);
    if (!file_A) {
        goto clean_dir_b;
    }
    // Link from directory A to file A
    const char link_A_name[] = "alink";
    const char* link_A = mktestlink(dir_A, link_A_name, file_A);
    if (!link_A) {
        goto clean_file_a;
    }

    // File in directory B
    const char cont_B[] = "Every Villain is Lemons";
    const char* file_B = mktestfile(dir_B, name, cont_B);
    if (!file_B) {
        goto unlink_link_A;
    }
    const char link_B_name[] = "blink";
    // Link from directory A to file B in directory B
    const char* link_B = mktestlink(dir_A, link_B_name, file_B);
    if (!link_B) {
        goto unlink_file_B;
    }

    // TESTS
    // The size of the file is used as a proxy for checking that stat worked.
    int dir_a_fd = open(dir_A, O_DIRECTORY);
    if (dir_a_fd == -1) {
        perror("open (dir A)");
        goto unlink_link_B;
    }

    // fstatat works (basic)
    size_t len_cont_A = strlen(cont_A);
    if (run_test(dir_a_fd, 0, name, file_A, len_cont_A) == -1) {
        goto close_dir_a_fd;
    }
    
    // fstatat follows symlinks (same dir)
    if (run_test(dir_a_fd, 0, link_A_name, link_A, len_cont_A) == -1) {
        fprintf(stderr, "Context: link %s -> %s\n", link_A, file_A);
        goto close_dir_a_fd;
    }

    // fstatat follows symlinks (to diff dir)
    size_t len_cont_B = strlen(cont_B);
    if (run_test(dir_a_fd, 0, link_B_name, link_B, len_cont_B) == -1) {
        fprintf(stderr, "Context: link %s -> %s\n", link_B, file_B);
        goto close_dir_a_fd;
    }

    // TODO: AT_SYMLINK_NOFOLLOW (no Redox support)

    // TODO: O_SEARCH (no Redox support)

    // AT_FDCWD
    char old_cwd[PATH_MAX] = {0};
    if (!getcwd(old_cwd, PATH_MAX)) {
        perror("getcwd");
        goto close_dir_a_fd;
    }
    if (chdir(dir_A) == -1) {
        perror("chdir");
        goto close_dir_a_fd;
    }
    if (run_test(AT_FDCWD, 0, name, "./", len_cont_A) == -1) {
        fputs("Context: AT_FDCWD\n", stderr);
        goto close_dir_a_fd;
    }
    if (chdir(old_cwd) == -1) {
        perror("chdir");
        goto close_dir_a_fd;
    }

    // Absolute path
    if (run_test(dir_a_fd, 0, file_A, file_A, len_cont_A) == -1) {
        fprintf(stderr, "Context: absolute path %s\n", file_A);
        goto close_dir_a_fd;
    }

    // Relative path that traverses dir boundaries
    // The path isn't resolved beneath the directory but rather resolved
    // relative to it. Directory traversal is allowed if AT_RESOLVE_BENEATH
    // isn't used.
    const char nested_name[] = "nested";
    char* nested_dir = malloc(strlen(dir_A) + strlen(nested_name) + 2);
    if (!nested_dir) {
        perror("malloc");
        goto close_dir_a_fd;
    }
    strcpy(nested_dir, dir_A);
    strcat(nested_dir, "/");
    strcat(nested_dir, nested_name);
    if (mkdir(nested_dir, 0700) == -1) {
        perror("mkdir");
        goto clean_nested_dir;
    }
    
    int nested_fd = open(nested_dir, O_DIRECTORY);
    if (nested_fd < 0) {
        perror("open");
        goto remove_nested_dir;
    }
    char rel_nested[sizeof(name) + 5] = "../";
    strcat(rel_nested, name);
    if (run_test(nested_fd, 0, rel_nested, rel_nested, len_cont_A) == -1) {
        fprintf(
            stderr,
            "Context: relative path from %s to ../%s\n",
            nested_dir,
            name
        );
        goto close_nested_dir;
    }

    // TODO: AT_RESOLVE_BENEATH

    // TODO: AT_EMPTY_PATH

    // TODO: Swapped directories resolves correctly

    // Failure conditions:
    // Empty path
    struct stat stat = {0};
    if (fstatat(dir_a_fd, "", &stat, 0) == 0) {
        fputs("Context: empty path should fail\n", stderr);
        goto close_dir_a_fd;
    }

    // Clean up is LIFO where the last created resource is the first cleaned.
    status = EXIT_SUCCESS;
close_nested_dir:
    close(nested_fd);
remove_nested_dir:
    rmdir(nested_dir);
clean_nested_dir:
    free((void*) nested_dir);
close_dir_a_fd:
    close(dir_a_fd);
unlink_link_B:
    unlink(link_B);
    free((void*) link_B);
unlink_file_B:
    unlink(file_B);
    free((void*) file_B);
unlink_link_A:
    unlink(link_A);
    free((void*) link_A);
clean_file_a:
    unlink(file_A);
    free((void*) file_A);
clean_dir_b:
    rmdir(dir_B);
clean_dir_a:
    rmdir(dir_A);
bye:
    return status;
}
