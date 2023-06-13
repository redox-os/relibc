#ifndef _BITS_SIGNAL_H
#define _BITS_SIGNAL_H

#define SIG_DFL ((void (*)(int))0)
#define SIG_IGN ((void (*)(int))1)
#define SIG_ERR ((void (*)(int))-1)

struct sigaction {
  union {
    void (*sa_handler)(int);
    void (*sa_sigaction)(int, siginfo_t *siginfo, void *context);
  };
  int sa_flags;
  sigset_t sa_mask;
  void (*sa_restorer)(void);
};

// XXX: cbindgen blocks both sigaction and struct sigaction, so define the function manually
int sigaction(int signal, const struct sigaction *restrict act, struct sigaction *restrict oldact);

#endif // _BITS_SIGNAL_H
