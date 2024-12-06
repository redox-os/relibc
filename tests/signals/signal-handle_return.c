#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

void SIGUSR1_handler(int signo)
{
	(void) signo;
	printf("do nothing useful\n");
}

void SIGUSR2_handler(int signo)
{
	(void) signo;
	printf("do nothing useful\n");
}

int main()
{
	if (signal(SIGUSR1, SIGUSR1_handler) == SIG_ERR) {
                perror("Unexpected error while using signal()");
               	exit(EXIT_FAILURE);
        }

	if (signal(SIGUSR2, SIGUSR2_handler) == SIG_ERR) {
                perror("Unexpected error while using signal()");
               	exit(EXIT_FAILURE);
        }

        if (signal(SIGUSR1,SIG_IGN) != SIGUSR1_handler) {
		printf("signal did not return the last handler that was associated with SIGUSR1\n");
               	exit(EXIT_FAILURE);
        }
    printf("test passed\n");
	return EXIT_SUCCESS;
} 