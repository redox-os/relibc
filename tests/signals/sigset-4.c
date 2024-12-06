#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

int signal_blocked = 0;
int count = 1;

void sig_handler(int signo)
{
	(void) signo;
	// printf("SIGCHLD called. Inside handler\n");
	sigset_t mask;
	sigprocmask(SIG_SETMASK, NULL, &mask);
    if (count == SIGKILL || count == SIGSTOP){
        count++;
    }
	if(sigismember(&mask, count)) {
		signal_blocked = 1;
	}
    count++;
}

int sigset_test4(int signum)
{
	if (sigset(signum, sig_handler) == SIG_ERR) {
                perror("Unexpected error while using sigset()");
               	exit(EXIT_FAILURE);
        }

	raise(signum);
	
	if (signal_blocked != 1) {
		printf("Test FAILED: signal was not added to the mask before the handler was executed\n");
		exit(EXIT_FAILURE);
	}
    signal_blocked = 0;	
    printf("test %d passed: the singal was added to the mask before the handler was executed\n", signum);
	return EXIT_SUCCESS;
} 

int main(){
    for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		sigset_test4(i);
	}
	return EXIT_SUCCESS;
}



