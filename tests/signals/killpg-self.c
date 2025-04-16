#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "signals_list.h"
#include "../test_helpers.h"

// Test that the killpg() function shall send signal sig to the process 
// group specified by prgp.

void sig_handler(int signo)
{
	// printf("Caught signal %d being tested!\n", signo);
	// printf("Test PASSED\n");
	(void) signo;
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

	pgrp = getpgrp();
	ERROR_IF(getpgrp, pgrp, == -1);

	status = killpg(pgrp, signum);
	ERROR_IF(killpg, status, != 0);

    return EXIT_SUCCESS;
}

int main(){
    // UB if pg == 1, so set it here first
    int status = setpgid(0, 0);
    ERROR_IF(setpgid, status, == -1);

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

