#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <sys/types.h>
#include "../test_helpers.h"

int main()
{

	/*
	 * ESRCH
	 */
	if (-1 == kill(999999, 0)) {
		if (ESRCH == errno) {
			printf("ESRCH error received\n");
		} else {
			printf("kill() failed on ESRCH errno not set correctly\n");
			exit(EXIT_FAILURE);
		}	
	} else {
		printf("kill() did not fail on ESRCH\n");
		exit(EXIT_FAILURE);
	}

	
	printf("test passed\n");
}