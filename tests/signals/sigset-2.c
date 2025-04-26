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
	printf("SIGUSR1 called. Inside handler\n");
	handler_called = 1;
}

int sigset_test2(int signum)
{
	struct sigaction act;
	act.sa_flags = 0;
	act.sa_handler = sig_handler;
	sigemptyset(&act.sa_mask);

	if (sigaction(signum, &act, 0) != 0) {
                perror("Unexpected error while using sigaction()");
               	exit(EXIT_FAILURE);
        }

        if (sigset(signum,SIG_IGN) != sig_handler) {
                perror("Unexpected error while using signal()");
               	exit(EXIT_FAILURE);
        }

	raise(signum);
	
	if (handler_called == 1) {
		printf("Test FAILED: handler was called even though ignore was expected\n");
		exit(EXIT_FAILURE);
	}		
    printf("test %d passed, the signal was ignored\n", signum);
	return EXIT_SUCCESS;
} 

int main(){
    for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		sigset_test2(i);
	}
	return EXIT_SUCCESS;
}

