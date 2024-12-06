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

	if(pthread_create(&new_th, NULL, a_thread_func, (void *)&signum) != 0)
	{
		perror("Error creating thread\n");
		exit(EXIT_FAILURE);
	}

	sleep(1);

	if(pthread_kill(new_th, signum) != 0) 
	{
		printf("Test UNRESOLVED: Couldn't send signal to thread\n");
		exit(EXIT_FAILURE);
	}

	sleep(1);

	if(handler_called != 1) {
		printf("Test FAILED: signal wasn't removed from signal mask\n");
		exit(EXIT_FAILURE);
	}
	handler_called = 0;

	printf("Test PASSED\n");
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
