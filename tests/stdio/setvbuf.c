#include <stdio.h>
#include <unistd.h>
#include <sys/wait.h>
#include <stdlib.h>

void test(int flag, char* line1, char* line2) {
    pid_t runner = fork();
    if (runner == 0) {
        fflush(stdout);
        setvbuf(stdout, NULL, flag, 0);

        printf("never gonna ");
        pid_t pid = fork();
        if (pid == 0) {
            printf("%s\n", line1);
        } else {
            wait(NULL);
            printf("%s\n", line2); 
        }
        _exit(0);
    } else {
        wait(NULL);
    }
}

int main(void) {
    test(_IOFBF, "now you see me", "now you don't");
    test(_IOLBF, "give you up", "let you down");
    test(_IONBF, "tell a lie", "and desert you");
    return 0;
}
