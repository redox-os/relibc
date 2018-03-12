#include <sys/wait.h>
#include <unistd.h>
#include <stdlib.h>

int main(int argc, char** argv) {
    pid_t pid = fork();
    if (pid == 0) {
    // child
    sleep(1);
    exit(0);
} else {
    // parent
    int* stat_loc;
    waitpid(pid, stat_loc, 0);
    }
}
