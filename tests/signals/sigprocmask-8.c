#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

int handler_called = 0;
int sigprocmask_return_val = 1; /* some value that's not a 1 or 0 */

void sig_handler(int signo)
{
	(void) signo;
	handler_called = 1;
	if (sigprocmask_return_val != 1) {
		printf("FAIL: sigprocmask() returned before signal was delivered.\n");
		exit(EXIT_FAILURE);
	}
}

int main()
{
	struct sigaction act;
	sigset_t blocked_set1;
	sigemptyset(&blocked_set1);
	sigaddset(&blocked_set1, SIGABRT);

	act.sa_handler = sig_handler;
	act.sa_flags = 0;
	sigemptyset(&act.sa_mask);

	if (sigaction(SIGABRT,  &act, 0) == -1) {
		perror("Unexpected error while attempting to setup test "
		       "pre-conditions");
		exit(EXIT_FAILURE);
	}

	if (sigprocmask(SIG_SETMASK, &blocked_set1, NULL) == -1) {
		perror("Unexpected error while attempting to use sigprocmask.\n");
		exit(EXIT_FAILURE);
	}

	if ((raise(SIGABRT) == -1)) {
		perror("Unexpected error while attempting to setup test "
		       "pre-conditions");
		exit(EXIT_FAILURE);
	}

	sigprocmask_return_val = sigprocmask(SIG_UNBLOCK, &blocked_set1, NULL);

	if (sigprocmask_return_val != 0) {
		perror("Unexpected error while attempting to use sigprocmask.\n");
		exit(EXIT_FAILURE);
	}
	
	if (handler_called != 1) {
		perror("Handler wasn't called, implying signal was not delivered.\n");
		exit(EXIT_FAILURE);
	}	

	printf("Test PASSED: signal was delivered before the call to sigprocmask returned.\n");
	return EXIT_SUCCESS;
}