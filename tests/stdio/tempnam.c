#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "test_helpers.h"

static void test_prefix(const char *prefix);
static void test_dir(const char *dir);
static void test_dir_and_prefix(const char *dir, const char *prefix);

int main(void) {
    char *first_null = tempnam(NULL, NULL);
    if(first_null == NULL) {
	// NOTE: assuming that we can at least get one file name
	puts("tempnam(NULL, NULL) returned NULL on first try");
	exit(EXIT_FAILURE);
    }
    printf("%s\n", first_null);

    char *second_null = tempnam(NULL, NULL);
    if(second_null == NULL) {
	// NOTE: assuming that we can at least get one file name
	puts("tempnam(NULL, NULL) returned NULL on second try");
	free(first_null);
	exit(EXIT_FAILURE);
    }
    printf("%s\n", second_null);

    free(first_null);
    free(second_null);

    if(first_null == second_null) {
	puts("tempnam(NULL, NULL) returns the same address");
	exit(EXIT_FAILURE);
    }

    // Ensure the "prefix" argument works
    test_prefix("this_is_a_test_prefix");
    test_prefix("exact");
    test_prefix("hi");
    test_prefix("");

    // Ensure the "dir" argument works

    // NOTE: needed because TMPDIR is the first directory checked
    unsetenv("TMPDIR");

    test_dir("/tmp");
    test_dir("");
    // NOTE: assumes /root is NOT writable
    test_dir("/root");

    // Ensure "prefix" and "dir" work together
    test_dir_and_prefix("/tmp", "this_is_a_prefix");
    test_dir_and_prefix("/tmp", "exact");
    test_dir_and_prefix("/root", "exact");
    test_dir_and_prefix("/root", "long_prefix");
    test_dir_and_prefix("", "prefix");
    test_dir_and_prefix("/tmp", "test");

    return 0;
}

static void test_prefix(const char *prefix) {
    test_dir_and_prefix(NULL, prefix);
}

static void test_dir(const char *dir) {
    test_dir_and_prefix(dir, NULL);
}

static void test_dir_and_prefix(const char *dir, const char *prefix) {
    char funcbuf[256];
    if(dir && prefix) {
        snprintf(funcbuf, sizeof(funcbuf), "tempnam(\"%s\", \"%s\")", dir, prefix);
    } else if(dir) {
        snprintf(funcbuf, sizeof(funcbuf), "tempnam(\"%s\", NULL)", dir);
    } else if(prefix) {
        snprintf(funcbuf, sizeof(funcbuf), "tempnam(NULL, \"%s\")", prefix);
    } else {
        strcpy(funcbuf, "tempnam(NULL, NULL)");
    }

    char *result = tempnam(dir, prefix);
    if(!result) {
        printf("%s failed\n", funcbuf);
        exit(EXIT_FAILURE);
    }
    printf("%s\n", result);

    if(prefix) {
        char buf[7] = { '/' };
        strncpy(&buf[1], prefix, sizeof(buf) - 2);
        buf[6] = 0;

        char *prev = NULL;
        char *substr = result;
        do {
            prev = substr;
            substr = strstr(&substr[1], buf);
        } while(substr);
        substr = prev;

        if(!substr) {
            printf("%s did not add the full (5 bytes at most) prefix\n", funcbuf);
            free(result);
            exit(EXIT_FAILURE);
        } else if(strlen(substr) != strlen(&buf[1]) + L_tmpnam) {
            printf("%s has the wrong length\n", funcbuf);
            free(result);
            exit(EXIT_FAILURE);
        }
    }

    if(dir) {
        char *substr = strstr(result, dir);
        char *other_substr = strstr(result, P_tmpdir);
        if(!substr && !other_substr) {
            printf("%s is in an unexpected directory\n", funcbuf);
            free(result);
            exit(EXIT_FAILURE);
        }
    }

    free(result);
}

