
#define _XOPEN_SOURCE 600

#include <stdio.h>
#include <signal.h>
#include <stdlib.h>
#include "../test_helpers.h"

// sigprocmask( ) shall return 0, Upon successful completion; otherwise, it shall return -1
// and errno shall be set to indicate the error, and the process' signal mask shall be unchanged.

int main()
{

	sigset_t set;
	sigaddset (&set, SIGABRT);

	int status;
	status = sigprocmask(SIG_SETMASK, &set, NULL);
	ERROR_IF(sigprocmask, status, != 0);
	return EXIT_SUCCESS;
}