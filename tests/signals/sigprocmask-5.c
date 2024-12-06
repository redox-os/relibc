#include <signal.h>
#include <stdio.h>
#include <stdlib.h>


#define NUMSIGNALS 25

int is_changed(sigset_t set, int sig) {
	
	int i;
	int siglist[] = {SIGABRT, SIGALRM, SIGBUS, SIGCHLD, 
		SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT, 
		SIGPIPE, SIGQUIT, SIGSEGV, 
		SIGTERM, SIGTSTP, SIGTTIN, SIGTTOU, 
		SIGUSR1, SIGUSR2, SIGPROF, SIGSYS, 
		SIGTRAP, SIGURG, SIGVTALRM, SIGXCPU, SIGXFSZ };

	if (sigismember(&set, sig) != 1) {
		return 1;
	}
	for (i=0; i<NUMSIGNALS; i++) {
		if ((siglist[i] != sig)) {
			if (sigismember(&set, siglist[i]) != 0) {
				return 1;
			}
		}
	}
	return 0;
}

int main() {
	sigset_t actl, oactl;

	sigemptyset(&actl);
	sigemptyset(&oactl);

	sigaddset(&actl, SIGABRT);

	sigprocmask(SIG_SETMASK, &actl, NULL);
	sigprocmask(SIG_BLOCK, NULL, &oactl);
	
	if (is_changed(oactl, SIGABRT)) {
		exit(EXIT_FAILURE);
	}
	printf("PASS: signal mask was not changed.\n");
	return EXIT_SUCCESS;
}