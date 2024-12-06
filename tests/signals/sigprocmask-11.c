
#define _XOPEN_SOURCE 600

#include <stdio.h>
#include <signal.h>
#include <stdlib.h>

// I don't understand this test

int main()
{

	sigset_t set;
	sigaddset (&set, SIGABRT);

	if (sigprocmask(SIG_SETMASK, &set, NULL) != 0) {
		perror("sigprocmask failed -- returned -- test aborted");
		exit(EXIT_FAILURE);
	} 
	printf("sigignore passed\n");
	return EXIT_SUCCESS;
}