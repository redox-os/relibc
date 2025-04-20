#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "../test_helpers.h"

// test sending signal 0 to self will killpg

int main()
{
    // UB if pg == 1, so set it here first
    int err = setpgid(0, 0);
    ERROR_IF(setpgid, err, == -1);

	int pgrp;

	pgrp = getpgrp();
	ERROR_IF(getpgrp, pgrp, == -1);

	int status;
	status = killpg(pgrp, 0);
	ERROR_IF(killpg, status, != 0);

	return EXIT_SUCCESS;
}
