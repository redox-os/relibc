#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include "../test_helpers.h"

// if a user tries to kill a process that does not exist the esrch error will be returned
int main()
{

	int status;
	status = killpg(999999, 0);
	ERROR_IF(killpg, status, !=-1);

	ERROR_IF(killpg, errno, != ESRCH);

	return EXIT_SUCCESS;
}