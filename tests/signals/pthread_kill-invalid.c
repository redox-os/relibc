#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <sys/types.h>

int main()
{
	pthread_t main_thread;

	main_thread = pthread_self();

	if (EINVAL != pthread_kill(main_thread, -1)) {
		printf("pthread_kill() did not fail on EINVAL\n");
		exit(EXIT_FAILURE);
	}
    printf("test pass\n");
	return 0;
}