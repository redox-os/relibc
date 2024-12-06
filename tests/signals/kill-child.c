#include <signal.h>
#include <sys/wait.h>
#include "signals_list.h"
#include "../test_helpers.h"

/*
 * Test signal catching when being signalled from parent.
 * Skip SIGKILL and SIGSTOP as these are not catchable.
 */

int sig_handled = 0;

void sig_handler(int signo)
{
	(void) signo;
	sig_handled = 1;
}

void child_proc(int signum)
{
	
	int sig;
	(void) sig;
	sigset_t sig_set;
	int status;

	sig_handled = 0;

	status = sigemptyset(&sig_set);
	ERROR_IF(sigemptyset, status, == -1);

	status = sigaddset(&sig_set, signum);
	ERROR_IF(sigaddset, status, == -1);

	struct sigaction act;
	act.sa_handler = sig_handler;
	act.sa_flags = 0;
	sigemptyset(&act.sa_mask);
	sigaction(signum, &act, NULL);

	status = sleep(10);
	ERROR_IF(sleep, status, == 0);

	if (sig_handled == 0)
	{
		printf("signal handler was not called");
		exit(EXIT_FAILURE);
	}

	exit(EXIT_SUCCESS);
}

void parent(int signum, pid_t pid)
{
	int status;

	sleep(1);
	status = kill(pid, signum);
	ERROR_IF(kill, status, != 0);

	pid = wait(&status);
	ERROR_IF(wait, pid, == (pid_t)-1);

	if (!WIFEXITED(status) || WEXITSTATUS(status) != EXIT_SUCCESS)
	{
		printf("Child did not exit normally.\n");
		exit(EXIT_FAILURE);
	}
	else
	{
		return;
	}
}

void kill_child(int signum)
{
	int pid;

	if ((pid = fork()) == 0)
	{
		child_proc(signum);
	}
	else
	{
		parent(signum, pid);
	}
}

int main()
{
	for (int i = 1; i < N_SIGNALS; i++)
	{
		if (i == SIGKILL || i == SIGSTOP)
		{
			continue;
		}
		kill_child(i);
	}
	return EXIT_SUCCESS;
}
