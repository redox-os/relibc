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
	if (sigemptyset(&act.sa_mask) == -1) {
		perror("Error calling sigemptyset\n");
		exit(EXIT_FAILURE);
	}
	if (sigaction(signum, &act, 0) == -1) {
		perror("Error calling sigaction\n");
		exit(EXIT_FAILURE);
	}

 	if ((pgrp = getpgrp()) == -1) {
		printf("Could not get process group number\n");
		exit(EXIT_FAILURE);
	}

 	if (killpg(pgrp, signum) != 0) {
		printf("Could not raise signal being tested\n");
		exit(EXIT_FAILURE);
	}

    return 0;
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

