#define _XOPEN_SOURCE 700

#include <pthread.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

int handler_called = 0;

void handler() {
	// printf("signal was called\n");
	handler_called = 1;
	return;
}

void *a_thread_func(void *sig)
{
	int signum = *(int *)sig;
	printf("%d !!!\n", signum);
	struct sigaction act;
	act.sa_flags = 0;
	act.sa_handler = handler;
	sigemptyset(&act.sa_mask);
	sigaction(signum, &act, 0);
	sighold(signum);
	sigpause(signum);

	return NULL;
}



int sigpause_basic(int signum)
{
	pthread_t new_th;
	int status;
	status = pthread_create(&new_th, NULL, a_thread_func, (void *)&signum);
	ERROR_IF(pthread_create, status, != 0);

	sleep(1);

	status = pthread_kill(new_th, signum);
	ERROR_IF(pthread_kill, status, != 0);

	sleep(1);

	ERROR_IF(pthread_kill, handler_called,, != 1);
	handler_called = 0;

	return EXIT_SUCCESS;	
}



int main(){
	for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		sigpause_basic(i);
	}
	return EXIT_SUCCESS;
}
