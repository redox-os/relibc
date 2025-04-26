#define _XOPEN_SOURCE 600

#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <stdint.h>
#include <stdlib.h>
#include "../test_helpers.h"

// An errno value of [EINVAL] shall be returned and the sigprocmask() function shall fail, if the value of 
// the how argument is not equal to one of the defined values.

int main(int argc, char *argv[])
{
	int signo;
	(void) signo;
	int r=rand();
	sigset_t set;

	if (argc < 2) {
        	printf("Usage:  %s [1|2|3|4]\n", argv[0]);
		exit(EXIT_FAILURE);
	}

	/*
		Various error conditions
	*/
	switch (argv[1][0]) {
		case '1':
			signo=-1;
			break;
		case '2':
			signo=-10000;
			break;
		case '3':
			signo=INT32_MIN+1;
			break;
		case '4':
			signo=INT32_MIN;
			break;
		default:
			printf("Usage:  %s [1|2|3|4]\n", argv[0]);
			exit(EXIT_FAILURE);
	}

	sigaddset(&set, SIGABRT);
	
	int status;
	status = sigprocmask(r, &set, NULL);
	ERROR_IF(sigprocmask, status, != -1);
	ERROR_IF(sigprocmask, errno, != EINVAL);
	// if (sigprocmask(r, &set, NULL) == -1) {
	// 	if (EINVAL == errno) {
	// 		printf ("errno set to EINVAL\n");
	// 		return EXIT_SUCCESS;
	// 	} else {
	// 		printf ("errno not set to EINVAL\n");
	// 		exit(EXIT_FAILURE);
	// 	}
	// }
	
	exit(EXIT_FAILURE);
}