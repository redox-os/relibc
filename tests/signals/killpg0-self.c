#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "../test_helpers.h"

// test sending signal 0 to self will killpg

int main()
{
	int pgrp;

	pgrp = getpgrp();
	ERROR_IF(getpgrp, pgrp, == -1);

	int status;
	status = killpg(pgrp, 0);
	ERROR_IF(killpg, status, != 0);

	return EXIT_SUCCESS;
}