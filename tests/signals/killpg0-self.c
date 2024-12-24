#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main()
{
	int pgrp;

	pgrp = getpgrp();
	ERROR_IF(getpgrp, pgrp, == -1);

	int status;
	status = killpg(pgrp, 0);
	ERROR_IF(killpg, status, != 0);


	printf("Test PASSED\n");
	return EXIT_SUCCESS;
}