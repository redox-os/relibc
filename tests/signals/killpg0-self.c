#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main()
{
	int pgrp;

 	if ((pgrp = getpgrp()) == -1) {
		printf("Could not get process group number\n");
		exit(EXIT_FAILURE);
	}

 	if (killpg(pgrp, 0) != 0) {
		printf("killpg did not return success.\n");
		exit(EXIT_FAILURE);
	}

	printf("Test PASSED\n");
	return 0;
}