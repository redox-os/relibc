#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

int handler_called = 0;

void sig_handler(int signo)
{
	(void) signo;
	printf("SIGURG called. Inside handler\n");
	handler_called = 1;
}

int sigset_test(int signum)
{

	struct sigaction act;
	act.sa_handler = sig_handler;
	act.sa_flags = 0;
	sigemptyset(&act.sa_mask);

	if (sigaction(signum, &act, 0) != 0) {
                perror("Unexpected error while using sigaction()");
               	exit(EXIT_FAILURE);
        }

        if (sigset(signum,SIG_DFL) != sig_handler) {
                perror("Unexpected error while using signal()");
               	exit(EXIT_FAILURE);
        }

	raise(signum);
	
	if (handler_called == 1) {
		printf("Test FAILED: handler was called even though default was expected\n");
		exit(EXIT_FAILURE);
	}	
    printf("test passed, the signal was ignored\n");	
	return EXIT_SUCCESS;
} 

int main(){
    sigset_test(SIGURG);
	return EXIT_SUCCESS;
}

