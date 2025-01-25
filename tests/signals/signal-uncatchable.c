#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include "../test_helpers.h"

//make sure you can't catch uncatchable signals

void sig_handler(int signo)
{
	(void) signo;
	printf("handler does nothing useful.\n");
}

int main()
{
	errno = -1;
	void (*status) (int);
	status = signal(SIGKILL, sig_handler);
	ERROR_IF(signal, status, != SIG_ERR);

	ERROR_IF(signal, errno, <= 0);

	return EXIT_SUCCESS;
} 