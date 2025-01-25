#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "../test_helpers.h"

void SIGUSR1_handler(int signo)
{
	(void) signo;
	printf("do nothing useful\n");
}

void SIGUSR2_handler(int signo)
{
	(void) signo;
	printf("do nothing useful\n");
}

int main()
{
	void (*status) (int);
	status = signal(SIGUSR1, SIGUSR1_handler);
	ERROR_IF(signal, status, == SIG_ERR);

	status = signal(SIGUSR2, SIGUSR2_handler);
	ERROR_IF(signal, status, == SIG_ERR);

	status = signal(SIGUSR1,SIG_IGN);
	// printf("status is %d\n", status);
	// printf("SIGUSR1_handler is %d\n", SIGUSR1_handler);
	ERROR_IF(signal, status, != SIGUSR1_handler);

	//this seems to be a weird comparison error

	return EXIT_SUCCESS;
} 