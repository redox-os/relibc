#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <glob.h>


typedef int (*err_func_t)(const char *epath, int eerrno);


static char *gl_offs_test_string = "<Saved gl_offs string>";


int glob_err(const char *epath, int eerrno) {
    printf("glob_err(\"%s\", \"%s\") -> %d\n", epath, strerror(eerrno), 0);
    return 0;
}

void glob_test(const char *pattern, int flags, err_func_t errfunc, glob_t *pglob) {
    printf("Pattern = %s\n", pattern);

    int retval = glob(pattern, flags, errfunc, pglob);

    if (retval != 0) {
        if (retval == GLOB_ABORTED) {
            puts("(ABORTED)\n");
        }
        else if (retval == GLOB_NOMATCH) {
            puts("(NOMATCH)\n");
        }
        else if (retval == GLOB_NOSPACE) {
            puts("(NOSPACE)\n");
        }
        else {
            printf(" (Unknown retval %d!)\n", retval);
        }
        return;
    }

    size_t gl_offs = flags & GLOB_DOOFFS ? pglob->gl_offs : 0;

    printf("(Matched %lu)", pglob->gl_pathc);

    if (flags & GLOB_DOOFFS) {
        printf(" (with %lu gl_offs)\n", gl_offs);
    }
    else {
        puts("");
    }

    for(unsigned int i = 0; i < (gl_offs + pglob->gl_pathc); i++) {
        if (!pglob->gl_pathv[i]) {
            printf("%d - NULL\n", i);
        }
        else {
            printf("%d - %s\n", i, pglob->gl_pathv[i]);
        }
    }
    printf("%s", "\n");
}

int main(void) {
    glob_t pglob = {0};

    glob_test("eample_dir/*", 0, NULL, &pglob);
    globfree(&pglob);

    glob_test("eample_dir/*", GLOB_ERR, glob_err, &pglob);
    globfree(&pglob);

    glob_test("./example_dir/*", 0, glob_err, &pglob);
    globfree(&pglob);

    glob_test("example_dir/*never*", 0, glob_err, &pglob);
    glob_test("example_dir/?-and*", GLOB_APPEND, glob_err, &pglob);
    globfree(&pglob);

    pglob.gl_offs = 4;
    glob_test("example_dir/*never*", GLOB_DOOFFS, glob_err, &pglob);
    pglob.gl_pathv[0] = gl_offs_test_string;
    glob_test("example_dir/?-and*", GLOB_DOOFFS | GLOB_APPEND, glob_err, &pglob);
    globfree(&pglob);

    glob_test("example_dir", GLOB_MARK, glob_err, &pglob);
    globfree(&pglob);
}
