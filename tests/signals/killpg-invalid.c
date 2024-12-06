#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include "../test_helpers.h"

int main()
{
	int pgrp;

 	if ((pgrp = getpgrp()) == -1) {
		printf("Could not get process group number\n");
		exit(EXIT_FAILURE);
	}

 	if (killpg(pgrp, -1) != -1) {
		printf("killpg did not return -1 even though it was passed an invalid signal number.");
		exit(EXIT_FAILURE);
	}

	if (errno != EINVAL) {
		printf("killpg did not set errno to EINVAL even though it was passed an invalid signal number.");
		exit(EXIT_FAILURE);
	}

	printf("Test PASSED\n");
	return 0;
}