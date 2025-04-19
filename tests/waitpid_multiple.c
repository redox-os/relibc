#include <assert.h>
#include <sys/wait.h>
#include <unistd.h>
#include <stdlib.h>

#include "test_helpers.h"

int main(void) {
    // Spawn two children, one with same pgid and one with pgid set to its own
    // pid, so that the one with different pgid completes first. Then test
    // waitpid both for 'any child' and for 'any child with same pgid'.

    pid_t pid_samepgid = fork();
    ERROR_IF(fork, pid_samepgid, == -1);

    if (pid_samepgid == 0) {
        // child
        sleep(2);
        _Exit(2);
    }
    pid_t pid_diffpgids[2];
    for (int i = 0; i < 2; i++) {
        pid_diffpgids[i] = fork();
        ERROR_IF(fork, pid_diffpgids[i], == -1);

        if (pid_diffpgids[i] == 0) {
            int ret = setpgid(0, 0);
            ERROR_IF(setpgid, ret, == -1);

            // child
            sleep(1);
            _Exit(i);
        }
    }
    int status;
    pid_t wid;

    // First, check that the first different-pgid proc is recognized.
    wid = waitpid(-1, &status, 0);
    ERROR_IF(waitpid, wid, == -1);
    assert(wid == pid_diffpgids[0]);
    assert(WIFEXITED(status));
    assert(WEXITSTATUS(status) == 0);

    // Then, check that the longest-waiting proc with the same pgid is properly matched.
    wid = waitpid(0, &status, 0);
    ERROR_IF(waitpid, wid, == -1);
    assert(wid == pid_samepgid);
    assert(WIFEXITED(status));
    assert(WEXITSTATUS(status) == 2);

    // Finally, the last same-pgid must have completed.
    wid = waitpid(-1, &status, WNOHANG);
    ERROR_IF(waitpid, wid, == -1);
    assert(wid == pid_diffpgids[1]);
    assert(WIFEXITED(status));
    assert(WEXITSTATUS(status) == 1);
}
