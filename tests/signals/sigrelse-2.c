#include <stdio.h>
#include <signal.h>
#include <stdlib.h>
#include "../test_helpers.h"

int main()
{
	int status;
	status = (int)sigrelse(SIGABRT);
	ERROR_IF(sigrelse, status, != 0);
	return EXIT_SUCCESS;
}