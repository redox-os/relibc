#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/wait.h>
#include "signals_list.h"
#include "../test_helpers.h"

void sig_handler (int signo) {
	(void) signo;
	exit(1);
}

int killpg_test2(int signum)
{
	int child_pid, child_pgid;
    printf("we are on signal %d\n   ", signum);

	if ((child_pid = fork()) == 0) {
		/* child here */
		struct sigaction act;
		act.sa_handler=sig_handler;
		act.sa_flags=0;
		sigemptyset(&act.sa_mask);
		sigaction(signum, &act, 0);

		/* change child's process group id */
		setpgrp();

		sigpause(SIGABRT);

		return 0;
	} else {
		/* parent here */
		int i;
		sigignore(signum);

		sleep(1);
		if ((child_pgid = getpgid(child_pid)) == -1) {
			printf("Could not get pgid of child\n");
			exit(EXIT_FAILURE);
		}


		if (killpg(child_pgid, signum) != 0) {
			printf("Could not raise signal being tested\n");
            exit(EXIT_FAILURE);
		}

		if (wait(&i) == -1) {
			perror("Error waiting for child to exit\n");
			exit(EXIT_FAILURE);
		}

		if (WEXITSTATUS(i)) {
			printf("Child exited normally\n");
			printf("Test PASSED\n");
			return 0;
		} else {
			printf("Child did not exit normally.\n");
			printf("Test FAILED\n");
			exit(EXIT_FAILURE);
		}
	}

	printf("Should have exited from parent\n");
	printf("Test FAILED\n");
	return EXIT_FAILURE;
}

int main(){
	int x;
	for (unsigned int i = 1; i < sizeof(signals_list)/sizeof(signals_list[0]); i++)
	{
		int sig = signals_list[i].signal;
		if (sig == SIGKILL || sig == SIGSTOP || sig == SIGCHLD)
		{
			continue;
		}
		x = killpg_test2(sig);
	}
	if (x == EXIT_FAILURE){
		return EXIT_FAILURE;
	} else {
		return EXIT_SUCCESS;
	}
}

