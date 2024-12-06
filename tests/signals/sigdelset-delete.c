

    // The sigdelset() function deletes the individual signal specified by signo from the signal set pointed to by set.

    // Applications should call either sigemptyset() or sigfillset() at least once for each object of type sigset_t prior to any other use of that object. If such an object is not initialized in this way, but is nonetheless supplied as an argument to any of pthread_sigmask(), sigaction(), sigaddset(), sigdelset(), sigismember(), sigpending(), sigprocmask(), sigsuspend(), sigtimedwait(), sigwait(), or sigwaitinfo(), the results are undefined.


#define _OPEN_SYS
#include <stdio.h>
#include <signal.h>
#include <unistd.h>
#include "signals_list.h"
#include "../test_helpers.h"



void delset_test(sigset_t *sigset, int signal){
  if (sigismember(sigset, signal) !=1){
    printf("the signal is already out of the set, %d\n", signal);
  }
  sigdelset(sigset, signal);
  if (sigismember(sigset, signal) ==0){
    printf("the signal was deleted successfully\n");
  }
  if (sigismember(sigset, signal) !=0){
    printf("the signal remove failed\n");
  }

}

int main() {
  sigset_t sigset;

  sigfillset(&sigset);

  for (int i = 1; i < N_SIGNALS; i++){
        delset_test(&sigset, i);
    }
  
}
