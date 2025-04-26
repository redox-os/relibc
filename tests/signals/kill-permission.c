#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <unistd.h>
#include <sys/types.h>
#include "../test_helpers.h"

// makes sure that a process is not killed if the user doesn't have permission to kill the process

int main(void)
{
	int status;
	// This is added in case user is root. If user is normal user, then it has no effect on the tests
	setuid(1000); 
	status = kill(1, 0);
	ERROR_IF(kill, status, != -1);
	ERROR_IF(kill, errno, != EPERM);
	return EXIT_SUCCESS;
}