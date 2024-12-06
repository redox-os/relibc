#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

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

	sigemptyset(&actl);
	sigemptyset(&oactl);
	sigaddset(&actl, SIGABRT);

	sigprocmask(SIG_SETMASK, &actl, NULL);

	sigaddset(&actl, SIGALRM);
	if (sigprocmask(r, &actl, NULL) != -1) {
		perror("sigprocmask() did not fail even though invalid how parameter was passed to it.\n");
		exit(EXIT_FAILURE);
	}

	sigprocmask(SIG_SETMASK, NULL, &oactl);

	if (sigismember(&oactl, SIGABRT) != 1) {
		printf("FAIL: signal mask was changed. \n");
		exit(EXIT_FAILURE);
	}

	if (is_changed(oactl)) {
		printf("FAIL: signal mask was changed. \n");
		exit(EXIT_FAILURE);
	}

	printf("PASS: signal mask was not changed.\n");
	return EXIT_SUCCESS;
}