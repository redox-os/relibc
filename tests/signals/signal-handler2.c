#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

int handler_called = 0;

void sig_handler(int signo)
{
	(void) signo;
	// printf("%d called. Inside handler\n", signo);
	handler_called = 1;
}

int signal_test3(int signum)
{
	void (*status) (int);
	status = signal(signum, sig_handler);
	ERROR_IF(signal, status, == SIG_ERR);
	
	raise(signum);
	
	ERROR_IF(raise, handler_called, != 1);
	return EXIT_SUCCESS;
}

int main(){
    for (unsigned int i = 0; i < sizeof(signals_list)/sizeof(signals_list[0]); i++){
		int sig = signals_list[i].signal;
		if (sig == SIGKILL || sig == SIGSTOP){
			continue;
		}
		signal_test3(sig);
	}
	return EXIT_SUCCESS;
}