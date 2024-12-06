#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

#define NUMSIGNALS 26

int is_empty(sigset_t *set) {

        int i;
        int siglist[] = {SIGABRT, SIGALRM, SIGBUS, SIGCHLD,
                SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT,
                SIGPIPE, SIGQUIT, SIGSEGV,
                SIGTERM, SIGTSTP, SIGTTIN, SIGTTOU,
                SIGUSR1, SIGUSR2, SIGPROF, SIGSYS,
                SIGTRAP, SIGURG, SIGVTALRM, SIGXCPU, SIGXFSZ };

        for (i=0; i<NUMSIGNALS; i++) {
		if (sigismember(set, siglist[i]) != 0)
			return 0;
        }
        return 1;
}

void sig_handler(int signo)
{
	printf("%d called. Inside handler\n", signo);
}

int sigset_test5(int signum)
{
	sigset_t mask;
	sigemptyset(&mask);

	sigprocmask(SIG_SETMASK, &mask, NULL);

	if (sigset(signum, sig_handler) == SIG_ERR) {
                perror("Unexpected error while using sigset()");
               	exit(EXIT_FAILURE);
        }

	raise(signum);
	sigprocmask(SIG_SETMASK, NULL, &mask);

	if (is_empty(&mask) != 1) {
		printf("Test FAILED: signal mask should be empty\n");
		exit(EXIT_FAILURE);
	}
    printf("sig %d was successfully removed from the mask when handler returned\n", signum);
	return EXIT_SUCCESS;
} 

int main(){
    for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		sigset_test5(i);
	}
	return 0;
}

