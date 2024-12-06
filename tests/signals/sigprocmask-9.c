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
		printf("FAIL: SIGKILL was added to the signal mask\n");
		exit(EXIT_FAILURE);
	} 
	if (sigismember(&set2, SIGSTOP)) {
		printf("FAIL: SIGSTOP was added to the signal mask\n");
		exit(EXIT_FAILURE);
	}
	if (sigprocmask_return_val == -1) {
		printf("FAIL: sigprocmask returned -1. System should be able to enforce blocking un-ignorable signals without causing sigprocmask() to return -1.\n");
		exit(EXIT_FAILURE);
	}

	printf("Test PASSED\n");
	return EXIT_SUCCESS;
}