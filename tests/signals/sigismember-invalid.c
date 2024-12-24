#define _OPEN_SYS
#include <stdio.h>
#include <signal.h>
#include <errno.h>
#include <stdlib.h>




int main() {
  sigset_t sigset;
  int status;

  sigfillset(&sigset);
  status = sigismember(&sigset, -1);
  ERROR_IF(sigismember, status, != -1);
  ERROR_IF(sigismember, errno, != EINVAL);

  return EXIT_SUCCESS;	

}