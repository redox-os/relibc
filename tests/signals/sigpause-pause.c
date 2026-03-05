#include <pthread.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

// This program verifies that sigpause() removes sig from the signal mask.

int handler_called = 0;

void handler() {
	handler_called = 1;
	return;
}

void *a_thread_func(void *sig)
{
	int status;
	int signum = *(int *)sig;
	printf("Pausing signal %s\n", signum);	
	struct sigaction act;
	act.sa_flags = 0;
	act.sa_handler = handler;
	status = sigemptyset(&act.sa_mask);
	ERROR_IF(sigemptyset, status, != 0);
	status = sigaction(signum, &act, 0);
	ERROR_IF(sigaction, status, != 0);
	status = sighold(signum);
	ERROR_IF(sighold, status, != 0);
	status = sigpause(signum);
	ERROR_IF(sigpause, status, != 0);

	return NULL;
}



int sigpause_basic(int signum)
{
	pthread_t new_th;
	int status;
	status = pthread_create(&new_th, NULL, a_thread_func, (void *)&signum);
	ERROR_IF(pthread_create, status, != 0);

	usleep(100000);

	status = pthread_kill(new_th, signum);
	ERROR_IF(pthread_kill, status, != 0);

	usleep(100000);

	if (handler_called != 1){
		prinft("handler wasn't called\n");
		exit(EXIT_FAILURE);
	}
	handler_called = 0;

	return EXIT_SUCCESS;	
}



int main(){
	for (int i=1; i<N_SIGNALS; i++){
		int sig = signals_list[i].signal;
		if (sig == SIGKILL || sig == SIGSTOP){
			continue;
		}
		sigpause_basic(sig);
	}
	return EXIT_SUCCESS;
}
