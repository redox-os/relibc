// The sigrelse() function shall remove sig from the signal mask of the calling process.

#include <sys/types.h>
#include <signal.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

#define _XOPEN_SOURCE 700

int handler_called = 0;

void sig_handler(int signo)
{
	(void) signo;
	handler_called = 1;
}

int sigrelse_test(int signum)
{
	struct sigaction act;
	
	act.sa_handler = sig_handler;
	act.sa_flags = 0;
	sigemptyset(&act.sa_mask);
	if (sigaction(signum,  &act, 0) == -1) {
		perror("Unexpected error while attempting to setup test "
		       "pre-conditions");
		exit(EXIT_FAILURE);
	}

	sighold(signum);
	
	if (raise(signum) == -1) {
		perror("Unexpected error while attempting to setup test "
		       "pre-conditions");
		exit(EXIT_FAILURE);
	}

	if (handler_called) {
		printf("UNRESOLVED. possible problem in sigrelse\n");
		exit(EXIT_FAILURE);
	}

	if (sigrelse(signum) == -1) {
		printf("UNRESOLVED. possible problem in sigrelse\n");
		exit(EXIT_FAILURE);
	}

	sleep(1);

	if (handler_called) {
		printf("PASS: %d successfully removed from signal mask\n", signum);
    handler_called = 0;
		return EXIT_SUCCESS;
	} 
	printf("FAIL\n");
	exit(EXIT_FAILURE);
	return EXIT_FAILURE;
}

int main(){
	for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		sigrelse_test(i);
	}
	return 0;
}

