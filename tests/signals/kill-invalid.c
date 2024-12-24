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
	int status;
	status = kill(999999, 0);
	ERROR_IF(kill, status, != -1);
	ERROR_IF(kill, errno, != ESRCH);
	return EXIT_SUCCESS;
}