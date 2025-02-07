#include <assert.h>
#include <sys/wait.h>
#include <signal.h>
#include <stdio.h>
#include <pthread.h>
#include <unistd.h>
#include <limits.h>
#include <errno.h>

#include "test_helpers.h"

#define THE_SIG SIGRTMIN

volatile sig_atomic_t num = 0;

int parent;

void validate(int sig, const siginfo_t *info)
{
  assert(sig == THE_SIG);
  assert(info != NULL);
  assert(info->si_signo == THE_SIG);
  assert(info->si_value.sival_int == num);
  assert(info->si_code == SI_QUEUE);
  assert(info->si_pid == parent);
}

void action(int sig, siginfo_t *info, void *context)
{
  (void)context;
  assert(context != NULL);
  validate(sig, info);
  num++;
}

int main(void)
{
  int status, fds[2];

  status = pipe(fds);
  ERROR_IF(pipe, status, == -1);

  parent = getpid();
  assert(parent != 0);

  sigset_t set, mask;
  status = sigfillset(&mask);
  ERROR_IF(sigfillset, status, == -1);
  status = sigdelset(&mask, SIGSEGV);
  ERROR_IF(sigdelset, status, == -1);
  status = sigdelset(&mask, SIGBUS);
  ERROR_IF(sigdelset, status, == -1);
  status = sigdelset(&mask, SIGILL);
  ERROR_IF(sigdelset, status, == -1);
  status = sigdelset(&mask, SIGFPE);
  ERROR_IF(sigdelset, status, == -1);
  status = sigdelset(&mask, SIGINT);
  ERROR_IF(sigdelset, status, == -1);
  status = sigprocmask(SIG_SETMASK, &mask, NULL);
  ERROR_IF(sigprocmask, status, == -1);

  status = sigemptyset(&set);
  ERROR_IF(sigemptyset, status, == -1);
  status = sigaddset(&set, THE_SIG);
  ERROR_IF(sigaddset, status, == -1);

  sigset_t empty_set;
  status = sigemptyset(&empty_set);
  ERROR_IF(sigemptyset, status, == -1);

  int child = fork();
  ERROR_IF(fork, child, == -1);

  status = close(fds[child == 0 ? 0 : 1]);
  ERROR_IF(close, status, == -1);

  struct sigaction sa;
  memcpy(&sa.sa_mask, &set, sizeof(sigset_t));
  sa.sa_flags = SA_SIGINFO;
  sa.sa_sigaction = action;

  status = sigaction(THE_SIG, &sa, NULL);
  ERROR_IF(sigaction, status, == -1);

  if (child == 0)
  {
    assert(num == 0);
    siginfo_t info;
    struct timespec t = (struct timespec){.tv_sec = 1, .tv_nsec = 200000000};
    status = sigtimedwait(&set, &info, &t);
    ERROR_IF(sigtimedwait, status, == -1);
    validate(THE_SIG, &info);
    assert(num == 0); // ensure no signal handler ran

    num++;

    // TODO: check status
    status = sigsuspend(&empty_set);
    if (status == -1)
    {
      UNEXP_IF(sigsuspend, errno, != EINTR);
    }

    assert(num == 2); // ensure signal handler ran

    status = sigprocmask(SIG_SETMASK, &empty_set, NULL);
    ERROR_IF(sigprocmask, status, == -1);

    while (num < 31)
    {
    }

    status = write(fds[1], "A", 1);
    ERROR_IF(write, status, == -1);
  }
  else
  {
    struct timespec t = (struct timespec){.tv_sec = 0, .tv_nsec = 100000000};
    status = nanosleep(&t, NULL);
    ERROR_IF(nanosleep, status, < 0);

    for (int n = 0; n <= 31; n++)
    {
      status = sigqueue(child, THE_SIG, (union sigval){.sival_int = n});
      ERROR_IF(sigqueue, status, == -1);
    }
    char buf[1];
    status = read(fds[0], buf, 1);
    ERROR_IF(read, status, == -1);

    pid_t wait_pid = 0;
    int wait_status = 0;
    wait_pid = wait(&wait_status);
    ERROR_IF(wait, wait_pid, < 0);
    UNEXP_IF(wait, wait_pid, != child);
    if (!WIFEXITED(wait_status) || WEXITSTATUS(wait_status) != EXIT_SUCCESS)
    {
      fprintf(stderr, "Unexpected result, WIFEXITED %s, WEXITSTATUS %d\n",
              WIFEXITED(wait_status) ? "true" : "false", WEXITSTATUS(wait_status));
      return EXIT_FAILURE;
    }
  }

  return EXIT_SUCCESS;
}
