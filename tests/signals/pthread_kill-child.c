#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>

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
	if (rc != 0)
	{
		printf("Error at pthread_create()\n");
		exit(EXIT_FAILURE);
	}
	
	rc = pthread_join(child_thread, NULL);
	if (rc != 0)
	{
		printf("Error at pthread_join()\n");
		exit(EXIT_FAILURE);
	}
		
	/* Now the child_thread exited, it is an invalid tid */
	memcpy(&invalid_tid, &child_thread, 
			sizeof(pthread_t)); 
    // int i = pthread_kill(invalid_tid, 0);
    sleep(3);
    // printf("%d\n", i);
    // printf("esrch is %d\n", ESRCH);
	
 	if (pthread_kill(invalid_tid, 0) == ESRCH) {
		printf("pthread_kill() returns ESRCH.\n");
		return 0;
	}

	printf("Test Fail\n");
	exit(1);
}