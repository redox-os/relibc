#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include "../test_helpers.h"

int main()
{

 	if (killpg(999999, 0) != -1) {
		printf("killpg did not return -1 even though it was passed an invalid process group id.");
		exit(EXIT_FAILURE);
	}

	if (errno != ESRCH) {
		printf("killpg did not set errno to ESRCH even though it was passed an invalid signal number.");
		exit(EXIT_FAILURE);
	}

	printf("Test PASSED\n");
	return 0;
}