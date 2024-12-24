#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "signals_list.h"
#include "../test_helpers.h"

void sig_handler(int signo)
{
	printf("Caught signal %d being tested!\n", signo);
	printf("Test PASSED\n");
	return;
}

int killpg_test1(int signum)
{
	int pgrp;
	struct sigaction act;

	act.sa_handler=sig_handler;
	act.sa_flags=0;
	int status = sigemptyset(&act.sa_mask);
	ERROR_IF(sigemptyset, status, == -1);

	status = sigaction(signum, &act, 0);
	ERROR_IF(sigaction, status, == -1);

	pgrp = getpgrp()
	ERROR_IF(getpgrp, pgrp, == -1);

	status = killpg(pgrp, signum);
	ERROR_IF(killpg, status, != 0);

    return EXIT_SUCCESS;
}

int main(){
	for (unsigned int i = 0; i < sizeof(signals_list)/sizeof(signals_list[0]); i++)
	{
		int sig = signals_list[i].signal;
		if (sig == SIGKILL || sig == SIGSTOP)
		{
			continue;
		}
		killpg_test1(sig);
	}
	return EXIT_SUCCESS;
}

