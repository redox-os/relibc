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

	int status;
	status = pthread_create(&new_th, NULL, b_thread_func, (void *)&signum);
	ERROR_IF(pthread_create, status, != 0);

	for (j=0; j<10; j++) {
		sleep(1);
		ERROR_IF(sigpuase, returned, == 1);
	}

	status = pthread_kill(new_th, signum);
	ERROR_IF(pthread_kill, status, != 0);

	sleep(1);

	ERROR_IF(sigpuase, returned, != 1);

	returned = 0;

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



