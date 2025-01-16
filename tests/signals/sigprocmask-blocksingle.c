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
	struct sigaction act;
	sigset_t blocked_set, pending_set;
	sigemptyset(&blocked_set);
	sigaddset(&blocked_set, signum);

	act.sa_handler = sig_handler;
	act.sa_flags = 0;
	sigemptyset(&act.sa_mask);

	int status;
	status = sigaction(signum,  &act, 0);
	ERROR_IF(sigaction, status, == -1);

	status = sigprocmask(SIG_SETMASK, &blocked_set, NULL);
	ERROR_IF(sigprocmask, status, == -1);

	status = raise(signum);
	ERROR_IF(raise, status, == -1);

	if (handler_called) {
		printf("FAIL: Signal was not blocked\n");
		exit(EXIT_FAILURE);
	}

	status = sigpending(&pending_set);
	ERROR_IF(sigpending, status, == -1);

	status = sigismember(&pending_set, signum);
	ERROR_IF(sigismember, status, != 1);

    act.sa_handler = SIG_IGN;
    sigaction(signum, &act, 0);

	return EXIT_SUCCESS;
}

int main(){
    for (int i=1; i<N_SIGNALS; i++){
		int sig = signals_list[i-1].signal;
		if (sig == SIGKILL || sig == SIGSTOP){
			continue;
		}
		sigprocmask_block(sig);
	}
	return EXIT_SUCCESS;
}

