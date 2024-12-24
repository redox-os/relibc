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

int kill_group(int signum)
{
	int pgrp;
	struct sigaction act;
	int status;

	act.sa_handler=sig_handler;
	act.sa_flags=0;
	status = sigemptyset(&act.sa_mask);
	ERROR_IF(sigemptyset, status, == -1);

	status = sigaction(signum, &act, 0);
	ERROR_IF(sigaction, staus, == -1);

	staus = getpgrp();
	ERROR_IF(getpgrp, status, == -1);

	status = kill(-pgrp, signum);
	ERROR_IF(kill, staus, != 0);

    return EXIT_SUCCESS;
}

// If pid is negative, but not -1, sig shall be sent to all processes (excluding an unspecified set of system processes) whose process group ID is equal to the absolute value of pid, and for which the process has permission to send a signal.
int main()
{
    for (unsigned int i = 0; i < sizeof(signals_list)/sizeof(signals_list[0]); i++)
	{
		int sig = signals_list[i].signal;
		if (sig == SIGKILL || sig == SIGSTOP)
		{
			continue;
		}
		kill_group(sig);
	}
	return EXIT_SUCCESS;
}