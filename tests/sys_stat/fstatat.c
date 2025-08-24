#include <assert.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <sys/stat.h>

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
    // Shouldn't ever fail.
    assert(strcpy(path, dir));
    assert(strcat(path, "/"));
    assert(strcat(path, name));

    int fd = creat(path, 0700);
    if (fd < 0) {
        perror("creat");
        free(path);
        return NULL;
    }

    size_t contents_len = strlen(contents);
    // Assumes that the few bytes of contents are fully written.
    if (write(fd, contents, contents_len) < 0) {
        perror("write");
        free(path);
        close(fd);
        return NULL;
    }

    close(fd);
    return path;
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

    const char name[] = "file";
    const char cont_A[] = "good";
    const char* file_a = mktestfile(dir_A, name, cont_A);
    if (!file_a) {
        goto clean_dir_b;
    }

    const char cont_B[] = "Every Villain is Lemons";
    const char* file_b = mktestfile(dir_B, name, cont_B);
    if (!file_b) {
        goto clean_file_a;
    }

    status = EXIT_SUCCESS;
clean_file_b:
    free((void*) file_b);
clean_file_a:
    free((void*) file_a);
clean_dir_b:
    rmdir(dir_B);
clean_dir_a:
    rmdir(dir_A);
bye:
    return status;
}
