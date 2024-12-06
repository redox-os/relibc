#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

int handler_called = 0;

void sig_handler(int signo)
{
	(void) signo;
	printf("SIGCHLD called. Inside handler\n");
	handler_called = 1;
}

int main()
{
	if (signal(SIGCHLD, sig_handler) == SIG_ERR) {
                perror("Unexpected error while using signal()");
               	exit(1);
        }

        if (signal(SIGCHLD,SIG_DFL) != sig_handler) {
                perror("Unexpected error while using signal()");
               	exit(1);
        }

	raise(SIGCHLD);
	
	if (handler_called == 1) {
		printf("Test FAILED: handler was called even though default was expected\n");
		exit(1);
	}		
    printf("test passed \n");
    handler_called = 0;
	return 0;
} 