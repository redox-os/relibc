#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

int handler_called = 0;

void sig_handler(int signo)
{
	(void) signo;
	handler_called = 1;
}

int sigprocmask_block(int signum)
{
    int defaultsig = SIGALRM;
	(void) defaultsig;
    if (signum == SIGALRM) {
        defaultsig = SIGHUP;
    }
	struct sigaction act;
	sigset_t blocked_set, pending_set;
	sigemptyset(&blocked_set);
	sigaddset(&blocked_set, signum);

	act.sa_handler = sig_handler;
	act.sa_flags = 0;
	sigemptyset(&act.sa_mask);

	if (sigaction(signum,  &act, 0) == -1) {
		perror("Unexpected error while attempting to setup test "
		       "pre-conditions");
		exit(EXIT_FAILURE);
	}

	if (sigprocmask(SIG_SETMASK, &blocked_set, NULL) == -1) {
		perror("Unexpected error while attempting to use sigprocmask.\n");
		exit(EXIT_FAILURE);
	}

    if (raise(signum) == -1) {
		perror("Unexpected error while attempting to setup test "
		       "pre-conditions");
		exit(EXIT_FAILURE);
	}

	if (handler_called) {
		printf("FAIL: Signal was not blocked\n");
		exit(EXIT_FAILURE);
	}

	if (sigpending(&pending_set) == -1) {
		perror("Unexpected error while attempting to use sigpending\n");
		exit(EXIT_FAILURE);
	}

	if (sigismember(&pending_set, signum) != 1) {
		perror("FAIL: sigismember did not return 1\n");
		exit(EXIT_FAILURE);
	}

	printf("Test PASSED: signal was added to the process's signal mask\n");
    act.sa_handler = SIG_IGN;
    sigaction(signum, &act, 0);

	return EXIT_SUCCESS;
}

int main(){
    for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		sigprocmask_block(i);
	}
	return EXIT_SUCCESS;
}

