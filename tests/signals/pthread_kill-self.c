#include <pthread.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>
#include <semaphore.h>
#include "signals_list.h"
#include "../test_helpers.h"

//test pthread_kill on self

sem_t sem_main;
sem_t sem_thread;
volatile sig_atomic_t handler_called = 0;

struct signal {
	int signum;
};

void handler(int sig) {
	(void) sig;
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

	sem_post(&sem_main);
	sem_wait(&sem_thread);
	// pthread_kill here
	sem_wait(&sem_thread);

	// sleep(50);

	handler_called=-1;
	pthread_exit(0);
	return NULL;
}

int pthread_kill_test1(int signum)
{
	pthread_t new_th;

	sem_init(&sem_main, 0, 0);
	sem_init(&sem_thread, 0, 0);

	struct signal arg;
	arg.signum = signum;

	int status;
	status = pthread_create(&new_th, NULL, a_thread_func, &arg);
	ERROR_IF(pthread_create, status, != 0);

	sem_wait(&sem_main);

	status = pthread_kill(new_th, signum);
	ERROR_IF(pthread_kill, status, != 0);

	usleep(20000);
	sem_post(&sem_thread);
	
	while(handler_called==0)
		usleep(20000);

	ERROR_IF(pthread_kill, handler_called, == -1);
	ERROR_IF(pthread_kill, handler_called, == 0);
	pthread_join(new_th, NULL);

	sem_post(&sem_thread);
	sem_destroy(&sem_main);
	sem_destroy(&sem_thread);

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
