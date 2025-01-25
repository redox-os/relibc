#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "../test_helpers.h"

int handler_called = 0;

void sig_handler(int signo)
{
	(void) signo;
	printf("SIGCHLD called. Inside handler\n");
	handler_called = 1;
}

int main()
{
	void (*status) (int);
	status = signal(SIGCHLD, sig_handler);
	ERROR_IF(signal, status, == SIG_ERR);

	status = signal(SIGCHLD,SIG_DFL);
	ERROR_IF(signal, status, != sig_handler);

	//same comparison error from handle_return
    
	raise(SIGCHLD);
	
	if (handler_called == 1) {
		return EXIT_FAILURE;
	}
    handler_called = 0;
	return EXIT_SUCCESS;
} 