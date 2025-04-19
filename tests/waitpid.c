#include <assert.h>
#include <sys/wait.h>
#include <unistd.h>
#include <stdlib.h>

#include "test_helpers.h"

void for_code(int code) {
    pid_t pid = fork();
    ERROR_IF(fork, pid, == -1);

    // Testing successful exit
    if (pid == 0) {
        // child
        sleep(1);
        _Exit(code);
    }
    printf("Testing waitpid of child %d for code %d\n", pid, code);
    // parent
    int status;
    pid_t wid = waitpid(pid, &status, 0);
    ERROR_IF(waitpid, wid, == -1);

    assert(WIFEXITED(status));
    assert(WEXITSTATUS(status) == code);

    puts("Success");
}

int main(void) {
    for_code(EXIT_SUCCESS);
    for_code(EXIT_FAILURE);
    for_code(42);
    for_code(255);
    // TODO: Also add coverage for e.g. WIFSTOPPED, WSTOPSIG, WTERMSIG, etc
}
