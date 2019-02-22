#include <sys/wait.h>
#include <unistd.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    pid_t pid = fork();
    ERROR_IF(fork, pid, == -1);

    if (pid == 0) {
        // child
        sleep(1);
        exit(EXIT_SUCCESS);
    } else {
        // parent
        int stat_loc;
        pid_t wid = waitpid(pid, &stat_loc, 0);
        ERROR_IF(waitpid, wid, == -1);
    }
}
