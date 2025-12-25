#include <pthread.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

//test pthread_kill on self

# define INTHREAD 0
# define INMAIN 1
# define SIGTOTEST SIGABRT

int sem1;		/* Manual semaphore */
volatile sig_atomic_t handler_called = 0;
int count = 1;

struct signal {
	int signum;
};

void handler() {
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
		usleep(100);

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

	int status;
	status = pthread_create(&new_th, NULL, a_thread_func, &arg);
	ERROR_IF(pthread_create, status, != 0);

	while(sem1==INTHREAD)
		usleep(100);

	status = pthread_kill(new_th, signum);
	ERROR_IF(pthread_kill, status, != 0);

    usleep(200);
	sem1=INTHREAD;
	
	while(handler_called==0)
		usleep(100);

	ERROR_IF(pthread_kill, handler_called, == -1);
	ERROR_IF(pthread_kill, handler_called, == 0);
	
	handler_called = 0;
	return EXIT_SUCCESS;	
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

