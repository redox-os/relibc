#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <sys/ptrace.h>
#include <sys/user.h>
#include <sys/wait.h>
#include <unistd.h>

#include "test_helpers.h"

#ifdef __linux__

const int SYS_write = 1;

#endif
#ifdef __redox__

const int SYS_write = 0x21000004;

#endif

int main() {
    int pid = fork();
    ERROR_IF(fork, pid, == -1);

    if (pid == 0) {
        // Test behavior on Redox when TRACEME hasn't been activated
        // before waitpid is invoked!
        sleep(1);

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
        int result = waitpid(pid, NULL, 0);
        ERROR_IF(waitpid, result, == -1);

        int status;
        while (true) {
            puts("----- Pre-syscall -----");
            result = ptrace(PTRACE_SYSCALL, pid, NULL, NULL);
            ERROR_IF(ptrace, result, == -1);
            UNEXP_IF(ptrace, result, != 0);
            puts("Wait...");
            result = waitpid(pid, &status, 0);
            ERROR_IF(waitpid, result, == -1);
            if (WIFEXITED(status)) { break; }

            struct user_regs_struct regs;
            puts("Get regs");
            result = ptrace(PTRACE_GETREGS, pid, NULL, &regs);
            ERROR_IF(ptrace, result, == -1);

            if (regs.orig_rax == SYS_write || regs.orig_rax == SYS_write) {
                regs.rdi = 2;
                puts("Set regs");
                result = ptrace(PTRACE_SETREGS, pid, NULL, &regs);
                ERROR_IF(ptrace, result, == -1);
            }

            puts("Post-syscall");
            result = ptrace(PTRACE_SYSCALL, pid, NULL, NULL);
            ERROR_IF(ptrace, result, == -1);
            UNEXP_IF(ptrace, result, != 0);
            puts("Wait...");
            result = waitpid(pid, &status, 0);
            ERROR_IF(waitpid, result, == -1);
            if (WIFEXITED(status)) { break; }
        }
        printf("Child exited with status %d\n", WEXITSTATUS(status));
    }
}
