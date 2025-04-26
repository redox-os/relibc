#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

void sig_handler(int signo)
{
	(void) signo;
	printf("SIGUSR1 called. Inside handler\n");
}

int sigset_test9(int signum)
{
	struct sigaction act;
	act.sa_flags = 0;
	act.sa_handler = sig_handler;
	sigemptyset(&act.sa_mask);

	if (sigaction(signum, &act, 0) != 0) {
                perror("Unexpected error while using sigaction()");
               	exit(EXIT_FAILURE);
        }

        if (sigset(signum,SIG_DFL) != sig_handler) {
		printf("Test FAILED: sigset didn't return myhandler even though it was SIGUSR1's original disposition\n");
               	exit(EXIT_FAILURE);
        }
    printf("test %d passed \n", signum);
	return EXIT_SUCCESS;
} 

int main(){
    for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		sigset_test9(i);
	}
	return EXIT_SUCCESS;
}


