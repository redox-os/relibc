#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "../test_helpers.h"

// After the call to sigprocmask(), if there are any pending unblocked signals, at least one of those
// signals shall be delivered before the call to sigprocmask() returns.

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

	int status;

	status = sigaction(SIGABRT,  &act, 0);
	ERROR_IF(sigaction, status, == -1);

	status = sigprocmask(SIG_SETMASK, &blocked_set1, NULL);
	ERROR_IF(sigprocmask, status, == -1);

	status = raise(SIGABRT);
	ERROR_IF(raise, status, == -1); 

	sigprocmask_return_val = sigprocmask(SIG_UNBLOCK, &blocked_set1, NULL);

	ERROR_IF(sigprocmask, sigprocmask_return_val, != 0);

	ERROR_IF(raise, handler_called, != 1);	

	return EXIT_SUCCESS;
}