#include <unistd.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    // Constants specified in https://pubs.opengroup.org/onlinepubs/7908799/xsh/unistd.h.html
    printf("_POSIX_VERSION: %ld\n", _POSIX_VERSION);
    /* TODO:
    printf("_POSIX2_VERSION: %ld\n", _POSIX2_VERSION);
    printf("_POSIX2_C_VERSION: %ld\n", _POSIX2_C_VERSION);
    printf("_XOPEN_VERSION: %d\n", _XOPEN_VERSION);

    printf("_XOPEN_XCU_VERSION: %d\n", _XOPEN_XCU_VERSION);
    
    printf("_XOPEN_XPG2: %d\n", _XOPEN_XPG2);
    printf("_XOPEN_XPG3: %d\n", _XOPEN_XPG3);
    printf("_XOPEN_XPG4: %d\n", _XOPEN_XPG4);
    printf("_XOPEN_UNIX: %d\n", _XOPEN_UNIX);
    */

    printf("NULL: %p\n", NULL);

    printf("R_OK: %d\n", R_OK);
    printf("W_OK: %d\n", W_OK);
    printf("X_OK: %d\n", X_OK);
    printf("F_OK: %d\n", F_OK);

    printf("SEEK_SET: %d\n", SEEK_SET);
    printf("SEEK_CUR: %d\n", SEEK_CUR);
    printf("SEEK_END: %d\n", SEEK_END);

    // sysconf() constants (_SC_*) are tested separately

    printf("F_LOCK: %d\n", F_LOCK);
    printf("F_ULOCK: %d\n", F_ULOCK);
    printf("F_TEST: %d\n", F_TEST);
    printf("F_TLOCK: %d\n", F_TLOCK);

    // pathconf() constants (_PC_*) are tested separately

    printf("STDIN_FILENO: %d\n", STDIN_FILENO);
    printf("STDOUT_FILENO: %d\n", STDOUT_FILENO);
    printf("STDERR_FILENO: %d\n", STDERR_FILENO);

    return 0;
}
