#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <sys/ptrace.h>
#include <sys/user.h>
#include <sys/wait.h>
#include <unistd.h>

#include "test_helpers.h"

int main() {
    int pid = fork();
    ERROR_IF(fork, pid, == -1);

    if (pid == 0) {
        int result = ptrace(PTRACE_TRACEME, 0, NULL, NULL);
        ERROR_IF(ptrace, result, == -1);
        UNEXP_IF(ptrace, result, != 0);

        // Alert parent: I'm ready
        result = raise(SIGSTOP);
        ERROR_IF(raise, result, == -1);
        UNEXP_IF(raise, result, != 0);

        puts("This is printed to STDOUT.");
        puts("Or, at least, that's what I thought.");
        puts("But all write(...) syscalls are actually redirected to STDERR by the tracer.");
        puts("Big surprise, right!");
    } else {
        // Wait for child process to be ready
        int result = waitpid(pid, NULL, WUNTRACED);
        ERROR_IF(waitpid, result, == -1);

        int status;
        while (true) {
            // Pre-syscall:
            result = ptrace(PTRACE_SYSCALL, pid, NULL, NULL);
            ERROR_IF(ptrace, result, == -1);
            UNEXP_IF(ptrace, result, != 0);
            result = waitpid(pid, &status, 0);
            ERROR_IF(waitpid, result, == -1);
            if (WIFEXITED(status)) { break; }

            struct user_regs_struct regs;
            result = ptrace(PTRACE_GETREGS, pid, NULL, &regs);
            ERROR_IF(ptrace, result, == -1);

            if (regs.orig_rax == 1 || regs.orig_rax == 0x21000004) { // SYS_write on Redox and Linux
                regs.rdi = 2;
                result = ptrace(PTRACE_SETREGS, pid, NULL, &regs);
                ERROR_IF(ptrace, result, == -1);
            }

            // Post-syscall:
            result = ptrace(PTRACE_SYSCALL, pid, NULL, NULL);
            ERROR_IF(ptrace, result, == -1);
            UNEXP_IF(ptrace, result, != 0);
            result = waitpid(pid, &status, 0);
            ERROR_IF(waitpid, result, == -1);
            if (WIFEXITED(status)) { break; }
        }
        printf("Child exited with status %d\n", WEXITSTATUS(status));
    }
}
