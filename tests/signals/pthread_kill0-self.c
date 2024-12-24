#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main()
{
	pthread_t main_thread;

	main_thread=pthread_self();

	int status;
	status = pthread_kill(main_thread, 0);
	ERROR_IF(pthread_kill, status, != 0);

	return EXIT_SUCCESS;
}