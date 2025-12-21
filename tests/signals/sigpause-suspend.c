#include <assert.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>
#include "signals_list.h"
#include "../test_helpers.h"

// This program verifies that sigpause() suspends the calling process
//  until it receives a signal.

#define INMAIN 0
#define INTHREAD 1

volatile sig_atomic_t handler_called = 0;
volatile atomic_bool completed = false;

void handler() {
	handler_called = 1;
	return;
}
void *b_thread_func(void *code_raw) {
    int *code = code_raw;
	printf("Pausing signal %s\n", strsignal(*code));
	sigpause(*code);
    assert(handler_called != 0);
    *code = 0;
    atomic_store(&completed, true);

	return NULL;
}


int sigpause_suspend(int signum)
{
    atomic_store(&completed, false);
	int status;

	struct sigaction act;
	act.sa_flags = 0;
	act.sa_handler = handler;
	sigemptyset(&act.sa_mask);
	status = sigaction(signum, &act, 0);
    ERROR_IF(sigaction, status, == -1);

	pthread_t new_th;

    int code = signum;
	if ((status = pthread_create(&new_th, NULL, b_thread_func, (void *)&code)) != 0) {
        errno = status;
        perror("failed to create thread");
        return EXIT_FAILURE;
    }
    usleep(100);
    assert(!atomic_load(&completed));

	if ((status = pthread_kill(new_th, signum)) != 0) {
        errno = status;
        perror("failed to kill thread");
        return EXIT_FAILURE;
    }

    if ((status = pthread_join(new_th, NULL)) != 0) {
        errno = status;
        perror("failed to join thread");
        return EXIT_FAILURE;
    }

	assert(code == 0);
    assert(atomic_load(&completed));

	return EXIT_SUCCESS;
}


int main(){
    // TODO: upper limit for i (gives OOB otherwise)
	for (int i=1; i<31; i++){
		int sig = signals_list[i].signal;
		if (sig == SIGKILL || sig == SIGSTOP){
			continue;
		}
        printf("For signal %s (%d)\n", strsignal(sig), sig);
		sigpause_suspend(sig);
	}
	return EXIT_SUCCESS;
}



