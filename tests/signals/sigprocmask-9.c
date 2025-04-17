#include "../test_helpers.h"
#include <assert.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

//  Attempt to add SIGKILL and SIGSTOP to the process's signal mask and 
//  verify that:
//  - They do not get added.
//  - sigprocmask() does not return -1.

int main() {
	sigset_t set1, set2;
	int ret;

	sigemptyset(&set1);
	sigemptyset(&set2);
	sigaddset(&set1, SIGKILL);
	sigaddset(&set1, SIGSTOP);

	ret = sigprocmask(SIG_SETMASK, &set1, NULL);
    ERROR_IF(sigprocmask, ret, == -1);
	ret = sigprocmask(SIG_SETMASK, NULL, &set2);
    ERROR_IF(sigprocmask, ret, == -1);

	assert(!sigismember(&set2, SIGKILL));
	assert(!sigismember(&set2, SIGSTOP));
}
