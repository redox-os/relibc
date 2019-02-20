#include <errno.h>
#include <pwd.h>
#include <stdio.h>
#include <stdlib.h>

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
        return 1;
    }
    if (pwd != NULL) {
        print(pwd);
    }

    puts("--- Checking getpwnam ---");
    errno = 0;
    pwd = getpwnam("nobody");
    if (errno != 0) {
        perror("getpwnam");
        return 1;
    }
    if (pwd != NULL) {
        print(pwd);
    }

    puts("--- Checking getpwuid_r ---");
    struct passwd pwd2;
    struct passwd* result;
    char* buf = malloc(100);
    if (getpwuid_r(0, &pwd2, buf, 100, &result) < 0) {
        perror("getpwuid_r");
        free(buf);
        return 1;
    }
    if (result != NULL) {
        if (result != &pwd2) {
            free(buf);
            return 1;
        }
        print(&pwd2);
    }

    puts("--- Checking getpwnam_r ---");
    if (getpwnam_r("nobody", &pwd2, buf, 100, &result) < 0) {
        perror("getpwuid_r");
        free(buf);
        return 1;
    }
    if (result != NULL) {
        if (result != &pwd2) {
            free(buf);
            return 1;
        }
        print(&pwd2);
    }
    free(buf);

    puts("--- Checking getpwuid_r error handling ---");
    char buf2[1];
    if (getpwuid_r(0, &pwd2, buf2, 1, &result) == 0) {
        puts("This shouldn't have succeeded, but did!");
        return 1;
    }
    if (errno != ERANGE) {
        perror("getpwuid_r");
        return 1;
    }
    puts("Returned ERANGE because the buffer was too small 👍");
}
