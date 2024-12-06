#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>

void sig_handler(int signo)
{
	(void) signo;
	printf("handler does nothing useful.\n");
}

int main()
{
	errno = -1;

	if (signal(SIGKILL, sig_handler) != SIG_ERR) {
                printf("Test FAILED: signal() didn't return SIG_ERR even though a non-catchable signal was passed to it\n");
               	exit(EXIT_FAILURE);
        }

	if (errno <= 0) {
		printf("Test FAILED: errno wasn't set to a positive number even though a non-catchable signal was passed to the signal() function\n");
               	exit(EXIT_FAILURE);
        }
    printf("test passed\n");
	return EXIT_SUCCESS;
} 