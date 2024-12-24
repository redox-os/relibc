#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

int main() {

	sigset_t set1, set2;
	int sigprocmask_return_val = 1;

	sigemptyset(&set1);
	sigemptyset(&set2);
	sigaddset(&set1, SIGKILL);
	sigaddset(&set1, SIGSTOP);
	sigprocmask_return_val = sigprocmask(SIG_SETMASK, &set1, NULL);
	sigprocmask(SIG_SETMASK, NULL, &set2);


	if (sigismember(&set2, SIGKILL)) {
		exit(EXIT_FAILURE);
	} 
	if (sigismember(&set2, SIGSTOP)) {
		exit(EXIT_FAILURE);
	}
	if (sigprocmask_return_val == -1) {
		exit(EXIT_FAILURE);
	}

	return EXIT_SUCCESS;
}