#define _OPEN_SYS
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <stdlib.h>




int main() {
  sigset_t sigset;

  sigfillset(&sigset);
  if (sigismember(&sigset, -1)!=-1){
    printf("sigismember didn't return -1");
    exit(EXIT_FAILURE);
  } else if (EINVAL != errno) {
		printf("errno was not set to EINVAL\n");
		exit(EXIT_FAILURE);
	}

    printf ("errno set to EINVAL and sigismember returned -1\n");
	return 0;	

}