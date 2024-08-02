#include <assert.h>
#include <signal.h>
#include <stdio.h>
#include <pthread.h>
#include <unistd.h>
#include <limits.h>
#include <errno.h>

#include "test_helpers.h"

#define THE_SIG SIGRTMIN

volatile sig_atomic_t num = 1;

int parent;

void action(int sig, siginfo_t *info, void *context) {
  (void)context;
  assert(sig == THE_SIG);
  assert(info != NULL);
  assert(context != NULL);
  assert(info->si_signo == THE_SIG);
  assert(info->si_value.sival_int == num);
  assert(info->si_code == SI_QUEUE);
  assert(info->si_pid == parent);
  num++;
  write(1, "action\n", 7);
}

int main(void) {
  int status, fds[2];

  status = pipe(fds);
  ERROR_IF(pipe, status, == -1);

  parent = getpid();
  assert(parent != 0);

  int child = fork();
  ERROR_IF(fork, child, == -1);

  status = close(fds[child == 0 ? 0 : 1]);
  ERROR_IF(close, status, == -1);

  sigset_t set;
  status = sigfillset(&set);
  ERROR_IF(sigfillset, status, == -1);
  status = sigdelset(&set, SIGSEGV);
  ERROR_IF(sigdelset, status, == -1);
  status = sigdelset(&set, SIGBUS);
  ERROR_IF(sigdelset, status, == -1);
  status = sigdelset(&set, SIGILL);
  ERROR_IF(sigdelset, status, == -1);
  status = sigdelset(&set, SIGFPE);
  ERROR_IF(sigdelset, status, == -1);
  status = sigdelset(&set, SIGINT);
  ERROR_IF(sigdelset, status, == -1);
  status = sigprocmask(SIG_SETMASK, &set, NULL);
  ERROR_IF(sigprocmask, status, == -1);

  struct sigaction sa;
  memcpy(&sa.sa_mask, &set, sizeof (sigset_t));
  sa.sa_flags = SA_SIGINFO;
  sa.sa_sigaction = action;

  status = sigaction(THE_SIG, &sa, NULL);
  ERROR_IF(sigaction, status, == -1);

  if (child == 0) {
    status = sigemptyset(&set);
    ERROR_IF(sigemptyset, status, == -1);
    while (num != 32) {
    }
    status = write(fds[1], "A", 1);
    ERROR_IF(write, status, == -1);
  } else {
    for (int n = 1; n <= 32; n++) {
      status = sigqueue(child, THE_SIG, (union sigval){ .sival_int = n });
      ERROR_IF(sigqueue, status, == -1);
    }
    char buf[1];
    status = read(fds[0], buf, 1);
    ERROR_IF(read, status, == -1);
  }

  return 0;
}
