#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include "../test_helpers.h"

// the test makes sure that if an invalid signal is passed it will return the EINVAL error 
int main()
{
	int pgrp;

	pgrp = getpgrp();
	ERROR_IF(getpgrp, pgrp, == -1);
 	
	int status;
	status = killpg(pgrp, -1);
	ERROR_IF(killpg, status, != -1);

	ERROR_IF(killpg, errno, != EINVAL);

	return EXIT_SUCCESS;
}