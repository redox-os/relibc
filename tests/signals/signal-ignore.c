#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

int handler_called = 0;

void sig_handler(int signo)
{
	printf("%d called. Inside handler\n", signo);
	handler_called = 1;
}

int signal_test2(int signum)
{
	if (signal(signum, sig_handler) == SIG_ERR) {
                perror("Unexpected error while using signal()");
               	exit(EXIT_FAILURE);
        }

        if (signal(signum,SIG_IGN) != sig_handler) {
                perror("Unexpected error while using signal()");
               	exit(EXIT_FAILURE);
        }

	raise(signum);
	
	if (handler_called == 1) {
		printf("Test FAILED: handler was called even though ingore was expected\n");
		exit(EXIT_FAILURE);
	}		
    printf("test %d passed\n", signum);
    handler_called = 0;
	return EXIT_SUCCESS;
} 

int main(){
    for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		signal_test2(i);
	}
	return EXIT_SUCCESS;
}

