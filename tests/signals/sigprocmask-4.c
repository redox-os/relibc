
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

// The previous mask shall be stored in the location pointed to by oset, if the argument oset is not a null pointer.


#define NUMSIGNALS 26

int main()
{
	sigset_t oactl, tempset;
	int i, j, test_failed=0;

	int siglist[] = {SIGABRT, SIGALRM, SIGBUS, SIGCHLD, 
		SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT, 
		SIGPIPE, SIGQUIT, SIGSEGV, 
		SIGTERM, SIGTSTP, SIGTTIN, SIGTTOU, 
		SIGUSR1, SIGUSR2, SIGPROF, SIGSYS, 
		SIGTRAP, SIGURG, SIGVTALRM, SIGXCPU, SIGXFSZ };

	for (i=0; i<NUMSIGNALS; i++) {
		sigemptyset(&oactl);
		sigemptyset(&tempset);
		sigaddset(&tempset, siglist[i]);
		sigprocmask(SIG_BLOCK, &tempset, &oactl);
		if (i > 0) {
			for (j=0; j<i; j++) { 
				if (sigismember(&oactl, siglist[j]) != 1) {
					test_failed = 1;
				}
			}
		}
	}

	if (test_failed != 0) {
		exit(EXIT_FAILURE);
	}

	return EXIT_SUCCESS;
}