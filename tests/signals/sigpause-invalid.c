#define _XOPEN_SOURCE 700

#include "../test_helpers.h"
#include <pthread.h>
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>

//  This program verifies that sigpause() returns -1 and sets errno to EINVAL
//  if passed an invalid signal number.

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

int sigpause_invalid(){
	int return_value = 0;

	return_value = sigpause(-1);
	ERROR_IF(sigpause, return_value, != -1);
	ERROR_IF(sigpause, errno, != EINVAL);
	if (return_value == -1) {
		if (errno == EINVAL) {
			printf ("Test PASSED: sigpause returned -1 and set errno to EINVAL\n");
			return EXIT_SUCCESS;
		} else {
			printf ("Test FAILED: sigpause did not set errno to EINVAL\n");
			exit(EXIT_FAILURE);
		}
	} else {
		printf ("Test FAILED: sigpause did not return -1\n");
		if (errno == EINVAL) {
			printf ("Test FAILED: sigpause did not set errno to EINVAL\n");
		}
		exit(EXIT_FAILURE);
	}
	return EXIT_SUCCESS;

}

int main(){
	sigpause_invalid();
	return EXIT_SUCCESS;
}

