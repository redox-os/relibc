#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include "../test_helpers.h"

int main()
{

	int status;
	status = killpg(999999, 0);
	ERROR_IF(killpg, status, !=-1);

	ERROR_IF(killpg, errno, != ESRCH);

	return EXIT_SUCCESS;
}