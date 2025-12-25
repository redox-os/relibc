#define _XOPEN_SOURCE 700

#include <pthread.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

//  This program verifies that sigpause() returns -1 and sets errno to EINTR
//  when it returns.

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

void *d_thread_func(void *sig)
{

	int signum = *(int *)sig;
	printf("%d Pausing signal \n", signum);
	int return_value = 0;
	struct sigaction act;
	act.sa_flags = 0;
	act.sa_handler = handler;
	sigemptyset(&act.sa_mask);
	sigaction(signum, &act, 0);
	return_value = sigpause(signum);
	ERROR_IF(sigpause, return_value, != -1);
	ERROR_IF(sigpause, errno, != EINTR);
	result = 0;
	if (return_value == -1) {
		if (errno == EINTR) {
			printf ("Test PASSED: sigpause returned -1 and set errno to EINTR\n");
			result = 0;
		} else {
			printf ("Test FAILED: sigpause did not set errno to EINTR\n");
			result = 1;
		}
	} else {
		if (errno == EINTR) {
			printf ("Test FAILED: sigpause did not return -1\n");
		}
		printf ("Test FAILED: sigpause did not set errno to EINTR\n");
		printf ("Test FAILED: sigpause did not return -1\n");
		result = 1;

	}
	sem = INMAIN;
	return NULL;
}


int sigpause_error(int signum){
	pthread_t new_th;

	int status;
	status = pthread_create(&new_th, NULL, d_thread_func, (void *)&signum);
	ERROR_IF(pthread_create, status, != 0);

	usleep(100);

	status = pthread_kill(new_th, signum);
	ERROR_IF(pthread_kill, status, != 0);

	sem = INTHREAD;
	while (sem == INTHREAD)
		usleep(100);

	if(result == 2) {
		exit(EXIT_FAILURE);
	}
	if(result == 1) {
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
		sigpause_error(i);
	}
	return EXIT_SUCCESS;
}

