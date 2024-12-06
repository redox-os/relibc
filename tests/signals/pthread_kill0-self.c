#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main()
{
	pthread_t main_thread;

	main_thread=pthread_self();

	if (pthread_kill(main_thread, 0) != 0) {
		printf("Could not call pthread_kill with sig = 0\n");
		exit(EXIT_FAILURE);
	}

	printf("Test PASSED\n");
	return 0;
}