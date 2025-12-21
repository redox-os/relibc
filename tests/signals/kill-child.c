#include <assert.h>
#include <signal.h>
#include <sys/wait.h>
#include "signals_list.h"
#include "../test_helpers.h"

/*
 * Test signal catching when being signalled from parent.
 * Skip SIGKILL and SIGSTOP as these are not catchable.
 */

volatile sig_atomic_t sig_handled = 0;

void sig_handler(int signo)
{
	(void) signo;
	sig_handled = 1;
}

void child_proc(int signum)
{
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

	status = usleep(200);
	ERROR_IF(usleep, status, == 0);

	assert(sig_handled != 0);

	exit(EXIT_SUCCESS);
}

void parent(int signum, pid_t pid)
{
	int status;

	usleep(100);
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
	for (long unsigned int i = 1; i <  sizeof(signals_list)/sizeof(signals_list[0]); i++)
	{
		int sig = signals_list[i].signal;
		if (sig == SIGKILL || sig == SIGSTOP)
		{
			continue;
		}
        printf("Testing for signal %s (%d)\n", strsignal(sig), sig);
		kill_child(sig);
	}
	return EXIT_SUCCESS;
}
