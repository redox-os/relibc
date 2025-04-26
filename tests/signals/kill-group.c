#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "signals_list.h"
#include "../test_helpers.h"

// this test is to make sure that when a negative pid is supplied to kill, all the processes in that group will be killed

int handler_called = 0;
void sig_handler(int signo)
{
	(void) signo;
	handler_called = 1;
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
	ERROR_IF(sigaction, status, == -1);

	status = getpgrp();
	pgrp = status;
	ERROR_IF(getpgrp, status, == -1);

	status = kill(-pgrp, signum);
	ERROR_IF(kill, status, != 0);

	ERROR_IF(kill, handler_called, !=1);
	handler_called = 0;

    return EXIT_SUCCESS;
}

int main()
{
    // Ensure we don't kill what was already in the process group.
    int status = setpgid(0, 0);
    ERROR_IF(setpgid, status, == -1);

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
