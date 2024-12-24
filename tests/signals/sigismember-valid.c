    // The sigismember() function shall test whether the signal specified by signo is a member of the set pointed to by set.

    // Applications should call either sigemptyset() or sigfillset() at least once for each object of type sigset_t prior to any other use of that object. If such an object is not initialized in this way, but is nonetheless supplied as an argument to any of pthread_sigmask(), sigaction(), sigaddset(), sigdelset(), sigismember(), sigpending(), sigprocmask(), sigsuspend(), sigtimedwait(), sigwait(), or sigwaitinfo(), the results are undefined.



#define _OPEN_SYS
#include <stdio.h>
#include <signal.h>
#include "signals_list.h"
#include "../test_helpers.h"

void check_full(sigset_t set, int signum) {
  if (!sigismember(&set, signum)) {
    printf("%d was not added to the set", signum);
    exit(EXIT_FAILURE);
  }
    
}

void check_empty(sigset_t set, int signum) {
  printf("%d is ", signum);
    if (sigismember(&set, signum)) {
      printf("%d was not removed from the set", signum);
      exit(EXIT_FAILURE);
    }

}



int main() {
  sigset_t sigset;

  sigfillset(&sigset);
  for (int i=1; i<N_SIGNALS; i++){
    check_full(sigset, i);
  }

  sigemptyset(&sigset);
  for (int i=1; i<N_SIGNALS; i++){
    check_empty(sigset, i);
  }

}