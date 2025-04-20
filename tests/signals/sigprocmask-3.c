#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

// The resulting set shall be the union of the current set and the signal 
//  set pointed to by set, if the value of the argument how is SIG_BLOCK.

volatile sig_atomic_t handler_called = 0;

void sig_handler(int signo)
{
	(void) signo;
	handler_called = 1;
}

int sigprocmask_block(int signum)
{
    int defaultsig = SIGALRM;
    if (signum == SIGALRM) {
        defaultsig = SIGHUP;
    }
	struct sigaction act;
	sigset_t set1, set2, pending_set;
	sigemptyset(&set1);
	sigemptyset(&set2);
	sigaddset(&set1, defaultsig);
	sigaddset(&set2, signum);

	act.sa_handler = sig_handler;
	act.sa_flags = 0;
	sigemptyset(&act.sa_mask);

	int status;
	status = sigaction(signum,  &act, 0);
	ERROR_IF(sigaction, status, == -1);

	status = sigaction(defaultsig,  &act, 0);
	ERROR_IF(sigaction, status, == -1);	

	status = sigprocmask(SIG_SETMASK, &set1, NULL);
	ERROR_IF(sigprocmask, status, == -1);

	status = sigprocmask(SIG_UNBLOCK, &set2, NULL);
	ERROR_IF(sigprocmask, status, == -1);

    printf("Raising %s\n", strsignal(signum));
	status = raise(signum);
	ERROR_IF(raise, status, == -1);

	ERROR_IF(raise, handler_called, != 1);
    handler_called = 0;

    printf("Raising %s\n", strsignal(defaultsig));
	status = raise(defaultsig);
	ERROR_IF(raise, defaultsig, == -1);

	ERROR_IF(raise, handler_called, == 1);

	status = sigpending(&pending_set);
	ERROR_IF(sigpending, status, == -1);

	status = sigismember(&pending_set, defaultsig);
	ERROR_IF(sigismember, status, != 1);

	status = sigismember(&pending_set, signum);
	ERROR_IF(sigismemeber, status, != 0);

	// printf("Test PASSED: signal was added to the process's signal mask\n");
    act.sa_handler = SIG_IGN;
    sigaction(signum, &act, 0);
    sigaction(defaultsig, &act, 0);

	return EXIT_SUCCESS;
}

int main(){
    for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
        printf("Testing signal %s (%d)\n", strsignal(i), i);
		sigprocmask_block(i);
	}
	return 0;
}

