#include <pthread.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

# define INTHREAD 0
# define INMAIN 1
# define SIGTOTEST SIGABRT

int sem1;		/* Manual semaphore */
int handler_called = 0;
int count = 1;

struct signal {
	int signum;
};

void handler() {
	printf("signal was called\n");
	handler_called = 1;
	return;
}

void *a_thread_func( void *arg)
{
	
	struct sigaction act;
	act.sa_flags = 0;
	act.sa_handler = handler;
	sigemptyset(&act.sa_mask);
	sigaction(((struct signal *)arg)->signum, &act, 0);

	sem1=INMAIN;

	while(sem1==INMAIN)
		sleep(1);

	// sleep(50);

	handler_called=-1;
	pthread_exit(0);
	return NULL;
}

int pthread_kill_test1(int signum)
{
	pthread_t new_th;

	sem1=INTHREAD;

	struct signal arg;
	arg.signum = signum;

	if(pthread_create(&new_th, NULL, a_thread_func, &arg) != 0)
	{
		perror("Error creating thread\n");
		exit(EXIT_FAILURE);
	}

	while(sem1==INTHREAD)
		sleep(1);

	if(pthread_kill(new_th, signum) != 0) 
	{
		printf("Test FAILED: Couldn't send signal to thread\n");
		exit(EXIT_FAILURE);
	}
    sleep(2);
	sem1=INTHREAD;
	
	while(handler_called==0)
		sleep(1);

	if(handler_called == -1) {
		printf("Test FAILED: Kill request timed out\n");
		exit(EXIT_FAILURE);
	} else if (handler_called == 0) {
		printf("Test FAILED: Thread did not recieve or handle\n");
		exit(EXIT_FAILURE);
	}

	printf("Test PASSED for signal %d\n", signum);
    handler_called = 0;
	return 0;	
}

int main(){
	for (unsigned int i = 0; i < sizeof(signals_list)/sizeof(signals_list[0]); i++)
	{
		int sig = signals_list[i].signal;
		if (sig == SIGKILL || sig == SIGSTOP)
		{
			continue;
		}
		pthread_kill_test1(sig);
	}
	return EXIT_SUCCESS;
}

