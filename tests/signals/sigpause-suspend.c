#define _XOPEN_SOURCE 700

#include <pthread.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

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
void *b_thread_func(void *sig)
{
	int signum = *(int *)sig;
	printf("%d !!!\n", signum);
	struct sigaction act;
	act.sa_flags = 0;
	act.sa_handler = handler;
	sigemptyset(&act.sa_mask);
	sigaction(signum, &act, 0);
	sigpause(signum);
	returned = 1;

	return NULL;
}


int sigpause_suspend(int signum)
{
	pthread_t new_th;
	int j;

	if(pthread_create(&new_th, NULL, b_thread_func, (void *)&signum) != 0)
	{
		perror("Error creating thread\n");
		exit(EXIT_FAILURE);
	}

	for (j=0; j<10; j++) {
		sleep(1);
		if (returned == 1) {
			printf ("Test FAILED: sigpause returned before it received a signal\n");
			exit(EXIT_FAILURE);
		}
	}

	if(pthread_kill(new_th, signum) != 0) 
	{
		printf("Test UNRESOLVED: Couldn't send signal to thread\n");
		exit(EXIT_FAILURE);
	}
	sleep(1);

	if(returned != 1) {
		printf("Test FAILED: signal was sent, but sigpause never returned.\n");
		exit(EXIT_FAILURE);
	}

	returned = 0;

	printf("test passed\n");
	return EXIT_SUCCESS;
}


int main(){
	for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
		sigpause_suspend(i);
	}
	return EXIT_SUCCESS;
}



