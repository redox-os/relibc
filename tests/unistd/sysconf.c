#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#define SC(N) { \
	errno = 0; \
	printf("%s (%d): %ld (%d)\n", #N, _SC_ ## N, sysconf(_SC_ ## N), errno); \
}

int main(void) {
	SC(ARG_MAX);
	SC(CHILD_MAX);
	SC(CLK_TCK);
	SC(NGROUPS_MAX);
	SC(OPEN_MAX);
	SC(STREAM_MAX);
	SC(TZNAME_MAX);
	SC(VERSION);
	SC(PAGESIZE);
	SC(RE_DUP_MAX);
	SC(LOGIN_NAME_MAX);
	SC(TTY_NAME_MAX);
	SC(SYMLOOP_MAX);
	SC(HOST_NAME_MAX);
	return EXIT_SUCCESS;
}
