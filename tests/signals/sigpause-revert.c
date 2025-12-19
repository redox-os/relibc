#define _XOPEN_SOURCE 700

#include <assert.h>
#include <pthread.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdbool.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

//  This program verifies that sigpause() restores sig to the signal mask before
//  returning.

volatile sig_atomic_t handler_called = false;

void handler() {
    handler_called = true;
	return;
}

void *c_thread_func(void *sig)
{
	int signum = *(int *)sig;
	printf("%d !!!\n", signum);

	sigset_t pendingset;

	puts("before sigpause");

    assert(!handler_called);

	if ((sigpause(signum) != -1) || (errno != EINTR)) {
		puts("Test UNRESOLVED: sigpause didn't return -1 and/or didn't set errno correctly.");
        exit(2);
		return NULL;
	}
    assert(handler_called);
    handler_called = false;

	int status = raise(signum);
    ERROR_IF(raise, status, == -1);

    assert(!handler_called);

	sigpending(&pendingset);
	if (sigismember(&pendingset, signum) == 1) {
		puts("Test PASSED: signal mask was restored when sigpause returned.");
	}
	
	return NULL;

}

int sigpause_revert(int signum) {
	pthread_t new_th;
    int status;
	struct sigaction act;

    // Ensure thread inherits mask with signum blocked.
	status = sighold(signum);
    ERROR_IF(sighold, status, == -1);

	act.sa_flags = 0;
	act.sa_handler = handler;
	status = sigemptyset(&act.sa_mask);
    ERROR_IF(sigemptyset, status, == -1);
	status = sigaction(signum, &act, NULL);
    ERROR_IF(sigaction, status, == -1);

	if((status = pthread_create(&new_th, NULL, c_thread_func, (void *)&signum)) != 0)
	{
        errno = status;
		perror("Error creating thread");
		exit(EXIT_FAILURE);
	}

	usleep(100);
    
	if((status = pthread_kill(new_th, signum)) != 0)
	{
        errno = status;
		perror("Test UNRESOLVED: Couldn't send signal to thread");
		exit(EXIT_FAILURE);
	}
    if ((status = pthread_join(new_th, NULL)) != 0) {
        errno = status;
        perror("failed to join thread");
        return EXIT_FAILURE;
    }

	puts("Test PASSED");
	return EXIT_SUCCESS;
}



int main(){
	for (int i=1; i<N_SIGNALS; i++){
		if (i == SIGKILL || i == SIGSTOP){
			continue;
		}
        printf("Testing signal %s\n", strsignal(i));
		sigpause_revert(i);
	}
	return EXIT_SUCCESS;
}

