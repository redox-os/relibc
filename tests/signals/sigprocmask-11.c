
#define _XOPEN_SOURCE 600

#include <stdio.h>
#include <signal.h>
#include <stdlib.h>

// I don't understand this test

int main()
{

	sigset_t set;
	sigaddset (&set, SIGABRT);

	int status;
	status = sigprocmask(SIG_SETMASK, &set, NULL);
	ERROR_IF(sigprocmask, status, != 0);
	return EXIT_SUCCESS;
}