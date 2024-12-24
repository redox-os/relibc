#include <signal.h>
#include "signals_list.h"
#include "../test_helpers.h"

/*
 * Test signal catching when signalling self.
 * Ensure all signals can be caught (other than SIGKILL and SIGSTOP).
 */

int handler_called = 0;

void sig_handler(int sig)
{
	(void) sig;
	handler_called = 1;
}

int kill_self(int sig)
{
	struct sigaction act;
	int status;

	handler_called = 0;

	act.sa_handler = sig_handler;
	act.sa_flags = 0;

	status = sigemptyset(&act.sa_mask);
	ERROR_IF(sigemptyset, status, == -1);

	status = sigaction(sig, &act, NULL);
	ERROR_IF(sigaction, status, == -1);

	status = kill(getpid(), sig);
	ERROR_IF(kill, status, != 0);

	ERROR_IF(kill, handler_called, == 0);

	return EXIT_SUCCESS;
}

int main()
{

	for (unsigned int i = 1; i < sizeof(signals_list)/sizeof(signals_list[0]); i++)
	{
		int sig = signals_list[i].signal;
		if (sig == SIGKILL || sig == SIGSTOP)
		{
			continue;
		}
		kill_self(sig);
	}
	return EXIT_SUCCESS;
}
