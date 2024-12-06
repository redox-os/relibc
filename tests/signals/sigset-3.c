#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

int handler_called = 0;

void sig_handler(int signo)
{
	(void) signo;
	// printf("SIGCHLD called. Inside handler\n");
	handler_called = 1;
}

int sigset_test3(int signum)
{
	if (sigset(signum, sig_handler) == SIG_ERR) {
                perror("Unexpected error while using sigset()");
               		exit(EXIT_FAILURE);
        }

	raise(signum);
	
	if (handler_called != 1) {
		printf("Test FAILED: handler wasn't called even though it was expected\n");
			exit(EXIT_FAILURE);
	}		
    printf("test %d passed, handler was called\n", signum);
    handler_called = 0;
	return EXIT_SUCCESS;
} 

int main(){
    for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		sigset_test3(i);
	}
	return EXIT_SUCCESS;
}

