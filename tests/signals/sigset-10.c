#define _XOPEN_SOURCE 600

#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include "../test_helpers.h"

int main()
{
	void (*status) (int);
	status = sigset(SIGKILL,SIG_IGN);
	ERROR_IF(sigset, status, == SIG_ERR);
	ERROR_IF(sigset, errno, != EINVAL);
	// if (sigset(SIGKILL,SIG_IGN) == SIG_ERR) {
	// 	if (errno != EINVAL) {
	// 		printf("Test FAILED: sigset() returned SIG_ERR but didn't set errno to EINVAL\n");
	// 		exit(EXIT_FAILURE);
	// 	}
	// } else {
	// 	printf("Test FAILED: sigset() didn't return SIG_ERROR even though SIGKILL was passed to it\n");
	// 	exit(EXIT_FAILURE);
	// }
    // printf("test passed: error was set successfully\n");
	return EXIT_SUCCESS;
} 