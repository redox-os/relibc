#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <unistd.h>
#include <sys/types.h>
#include "../test_helpers.h"

int main()
{
        setuid(1); /* this is added incase user is root. If user is normal user, then it has no effect on the tests*/

	if (kill(1, 0) == -1) {
		if (EPERM == errno) {
			printf("EPERM error received\n");
		} else {
			printf("kill() failed on EPERM errno not set correctly\n");
			exit(EXIT_FAILURE);
		}	
	} else {
		printf("kill() did not fail on EPERM\n");
		exit(EXIT_FAILURE);
	}

    printf("test passed\n");
	return 0;
}