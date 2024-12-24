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

	int status;
	status = pthread_kill(main_thread, -1);
	ERROR_IF(pthread_kill, status, != EINVAL);

	return EXIT_SUCCESS;
}