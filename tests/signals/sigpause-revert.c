#define _XOPEN_SOURCE 700

#include <pthread.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

//  This program verifies that sigpause() restores sig to the signal mask before
//  returning.

#define INMAIN 0
#define INTHREAD 1

int handler_called = 0;
int returned = 0;
int return_value = 2;
int result = 2;
int sem = INMAIN;

void handler() {
	// printf("signal was called\n");
	handler_called = 1;
	return;
}

void *c_thread_func(void *sig)
{
	int signum = *(int *)sig;
	printf("%d !!!\n", signum);
	struct sigaction act;
	sigset_t pendingset;

	act.sa_flags = 0;
	act.sa_handler = handler;
	sigemptyset(&act.sa_mask);
	sigaction(signum, &act, 0);
	sighold(signum);
	printf("after sigpause\n");

	if ((sigpause(signum) != -1) || (errno != EINTR)) {
		printf ("Test UNRESOLVED: sigpause didn't return -1 and/or didn't set errno correctly.");
		return_value = 2;
		return NULL;
	}

	sleep(1);

	raise (signum);
	sigpending(&pendingset);
	if (sigismember(&pendingset, signum) == 1) {
		printf("Test PASSED: signal mask was restored when sigpause returned.");
		return_value = 0;
	}
	
	sem = INMAIN;
	return NULL;

}

int sigpause_revert(int signum){
	pthread_t new_th;

	if(pthread_create(&new_th, NULL, c_thread_func, (void *)&signum) != 0)
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

	sem = INTHREAD;
	while (sem == INTHREAD)
		sleep(1);
	
	if(handler_called != 1) {
		printf("Test UNRESOLVED: signal wasn't removed from signal mask\n");
		exit(EXIT_FAILURE);
	}

	if (return_value != 0) {
		exit(EXIT_FAILURE);
	}
	
	printf("Test PASSED\n");
	return EXIT_SUCCESS;
}



int main(){
	for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		sigpause_revert(i);
	}
	return EXIT_SUCCESS;
}

