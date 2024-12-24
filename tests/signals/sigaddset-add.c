

    // The sigaddset() function adds the individual signal specified by the signo to the signal set pointed to by set.

    // Applications shall call either sigemptyset() or sigfillset() at least once for each object of type sigset_t prior to any other use of that object. If such an object is not initialized in this way, but is nonetheless supplied as an argument to any of pthread_sigmask(), sigaction(), sigaddset(), sigdelset(), sigismember(), sigpending(), sigprocmask(), sigsuspend(), sigtimedwait(), sigwait(), or sigwaitinfo(), the results are undefined.

#define _OPEN_SYS
#include <stdio.h>
#include <signal.h>
#include <unistd.h>
#include "signals_list.h"
#include "../test_helpers.h"



void addset_test(sigset_t *sigset, int signal){
  int status; 
  status = sigismember(sigset, signal);
  ERROR_IF(sigismember, status, != 0);
  sigaddset(sigset, signal);

  status = sigismember(sigset, signal);
  ERROR_IF(sigismember, status, != 1);

}

int main() {
  sigset_t sigset;

  sigemptyset(&sigset);

for (int i = 1; i < N_SIGNALS; i++){
        addset_test(&sigset, i);
    }


}
