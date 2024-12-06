#define _XOPEN_SOURCE 700

#include <stdio.h>
#include <signal.h>
#include <stdlib.h>

int main()
{

	if ((int)sigrelse(SIGABRT) != 0) {
		perror("sigrelse failed -- returned -- test aborted");
		exit(EXIT_FAILURE);
	} 
	printf("sigrelse passed\n");
	return EXIT_SUCCESS;
}