#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include "../test_helpers.h"

//test with pthread_kill to kill a child process

void * thread_function(void *arg)
{
	/* Does nothing */
	(void) arg;
	pthread_exit((void*)0);
	
	/* To please some compilers */
	return NULL;
}

int main()
{
	pthread_t child_thread;
	pthread_t invalid_tid;
	
	int rc;

	rc = pthread_create(&child_thread, NULL, 
		thread_function, NULL);
	ERROR_IF(pthread_create, rc, != 0);
	
	rc = pthread_join(child_thread, NULL);
	ERROR_IF(pthread_join, rc, != 0);
		
	// Now the child_thread exited, it is an invalid tid
	memcpy(&invalid_tid, &child_thread, 
			sizeof(pthread_t)); 
    sleep(3);

	int status;
	status = pthread_kill(invalid_tid, 0);
	ERROR_IF(pthread_kill, status, != ESRCH);
 
	exit(EXIT_SUCCESS);
}