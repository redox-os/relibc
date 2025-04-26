#include <signal.h>
#include "../test_helpers.h"

/*
 * Send signal 0 to self. This just reports whether a signal can be sent.
 */

int main()
{
	int status = 0;

	status = kill(getpid(), 0);
	ERROR_IF(kill, status, != 0);

	return EXIT_SUCCESS;
}