// The sigrelse() function shall remove sig from the signal mask of the calling process.

#include <assert.h>
#include <sys/types.h>
#include <signal.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

#define _XOPEN_SOURCE 700

volatile sig_atomic_t handler_called = 0;

void sig_handler(int signo)
{
	(void) signo;
	handler_called = 1;
}

int sigrelse_test(int signum)
{
    // needs to be reset
    handler_called = 0;

	struct sigaction act;
	
	act.sa_handler = sig_handler;
	act.sa_flags = 0;
	sigemptyset(&act.sa_mask);

	int status;
	status = sigaction(signum,  &act, NULL);
	ERROR_IF(sigaction, status, == -1);

	status = sighold(signum);
    ERROR_IF(sighold, status, == -1);

	assert(handler_called == 0);

	status = raise(signum);
	ERROR_IF(raise, status, == -1);	

	assert(handler_called == 0);

	status = sigrelse(signum);
	ERROR_IF(sigrelse, status, == -1);

	assert(handler_called == 1);
	// if (handler_called) {
	// 	printf("PASS: %d successfully removed from signal mask\n", signum);
    // handler_called = 0;
	// 	return EXIT_SUCCESS;
	// } 
	// printf("FAIL\n");
	// exit(EXIT_FAILURE);
	return EXIT_SUCCESS;
}

int main(){
	for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
        printf("For signal %s\n", strsignal(i));
		sigrelse_test(i);
	}
	return 0;
}

