#include <errno.h>
#include <pwd.h>
#include <stdio.h>
#include <stdlib.h>

#include "test_helpers.h"

void print(struct passwd *pwd) {
    printf("pw_name: %s\n", pwd->pw_name);
    printf("pw_password: %s\n", pwd->pw_passwd);
    printf("pw_uid: %u\n", pwd->pw_uid);
    printf("pw_gid: %u\n", pwd->pw_gid);
    printf("pw_gecos: %s\n", pwd->pw_gecos);
    printf("pw_dir: %s\n", pwd->pw_dir);
    printf("pw_shell: %s\n", pwd->pw_shell);
}

int main(void) {
    puts("--- Checking getpwuid ---");
    errno = 0;
    struct passwd *pwd = getpwuid(0);
    if (errno != 0) {
        perror("getpwuid");
        exit(EXIT_FAILURE);
    }
    if (pwd != NULL) {
        print(pwd);
    }

    puts("--- Checking getpwnam ---");
    errno = 0;
    pwd = getpwnam("nobody");
    if (errno != 0) {
        perror("getpwnam");
        exit(EXIT_FAILURE);
    }
    if (pwd != NULL) {
        print(pwd);
    }

    puts("--- Checking getpwuid_r ---");
    struct passwd pwd2;
    struct passwd* result;
    char* buf = malloc(300);
    if (getpwuid_r(0, &pwd2, buf, 100, &result) < 0) {
        perror("getpwuid_r");
        free(buf);
        exit(EXIT_FAILURE);
    }
    if (result != NULL) {
        if (result != &pwd2) {
            free(buf);
            exit(EXIT_FAILURE);
        }
        print(&pwd2);
    }

    puts("--- Checking getpwnam_r ---");
    if (getpwnam_r("nobody", &pwd2, buf, 300, &result) < 0) {
        perror("getpwuid_r");
        free(buf);
        exit(EXIT_FAILURE);
    }
    if (result != NULL) {
        if (result != &pwd2) {
            free(buf);
            exit(EXIT_FAILURE);
        }
        print(&pwd2);
    }
    free(buf);

    puts("--- Checking getpwuid_r error handling ---");
    char buf2[1];
    if (getpwuid_r(0, &pwd2, buf2, 1, &result) == 0) {
        puts("This shouldn't have succeeded, but it did!");
        exit(EXIT_FAILURE);
    }
    if (errno != ERANGE) {
        perror("getpwuid_r");
        exit(EXIT_FAILURE);
    }
    puts("Returned ERANGE because the buffer was too small 👍");

    errno = 0;

    struct passwd *entry = NULL;
    for (int i = 1; entry = getpwent(); ++i) {
        int backup = errno;
        printf("--- getpwent #%d ---\n", i);
        if (backup != 0) {
            errno = backup;
            perror("getpwent");
            exit(EXIT_FAILURE);
        }
        print(entry);
    }
    puts("--- getpwent #1 (rewind) ---");
    setpwent();
    entry = getpwent();
    perror("getpwent");
    print(entry);

    endpwent();
}
