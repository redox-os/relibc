#include <sys/wait.h>
#include <unistd.h>
#include <stdlib.h>

int main(void) {
    pid_t pid = fork();
    if (pid == 0) {
        // child
        sleep(1);
        return EXIT_SUCCESS;
    } else {
        // parent
        int stat_loc;
        waitpid(pid, &stat_loc, 0);
    }
    return EXIT_SUCCESS;
}
