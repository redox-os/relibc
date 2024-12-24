#define _XOPEN_SOURCE 700

#include <stdio.h>
#include <signal.h>
#include <stdlib.h>

int main()
{
	int status;
	status = (int)sigrelse(SIGABRT);
	ERROR_IF(sigrelse, status, != 0);
	return EXIT_SUCCESS;
}