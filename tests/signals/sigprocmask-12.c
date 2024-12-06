#define _XOPEN_SOURCE 600

#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <stdint.h>
#include <stdlib.h>

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
	
	if (sigprocmask(r, &set, NULL) == -1) {
		if (EINVAL == errno) {
			printf ("errno set to EINVAL\n");
			return EXIT_SUCCESS;
		} else {
			printf ("errno not set to EINVAL\n");
			exit(EXIT_FAILURE);
		}
	}
	
	printf("sighold did not return -1\n");
	exit(EXIT_FAILURE);
}