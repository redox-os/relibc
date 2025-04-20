#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "../test_helpers.h"

// The thread's signal mask shall not be changed, if sigprocmask( ) fails.

#define NUMSIGNALS 24

int is_changed(sigset_t set) {
	
	int i;
	int siglist[] = {SIGALRM, SIGBUS, SIGCHLD, 
		SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT, 
		SIGPIPE, SIGQUIT, SIGSEGV, 
		SIGTERM, SIGTSTP, SIGTTIN, SIGTTOU, 
		SIGUSR1, SIGUSR2, SIGPROF, SIGSYS, 
		SIGTRAP, SIGURG, SIGVTALRM, SIGXCPU, SIGXFSZ };

	for (i=0; i<NUMSIGNALS; i++) {
		if (sigismember(&set, siglist[i]) != 0)
			return 1;
	}
	return 0;
}

int get_rand() {

	int r;
	r=rand();
	while ((r == SIG_BLOCK) || (r == SIG_SETMASK) || (r == SIG_UNBLOCK)) {
		r = get_rand();
	}
	return r;
}

int main() {
	
	int r=get_rand();
	sigset_t actl, oactl;
	int status;

	sigemptyset(&actl);
	sigemptyset(&oactl);
	status = sigaddset(&actl, SIGABRT);
	ERROR_IF(sigaddset, status, != 0);

	status = sigprocmask(SIG_SETMASK, &actl, NULL);
	ERROR_IF(sigprocmask, status, != 0);

	status = sigaddset(&actl, SIGALRM);
	ERROR_IF(sigaddset, status, != 0);
	
	status = sigprocmask(r, &actl, NULL);
	ERROR_IF(sigprocmask, status, != -1);

	status = sigprocmask(SIG_SETMASK, NULL, &oactl);
	ERROR_IF(sigprocmask, status, != 0);

	status = sigismember(&oactl, SIGABRT);
	ERROR_IF(sigismember, status, != 1);

	if (is_changed(oactl)) {
		exit(EXIT_FAILURE);
	}

	return EXIT_SUCCESS;
}