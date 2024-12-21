#ifndef _BITS_SIGNAL_H
#define _BITS_SIGNAL_H

#define SIG_DFL ((void (*)(int))0)
#define SIG_IGN ((void (*)(int))1)
#define SIG_ERR ((void (*)(int))-1)

typedef struct siginfo siginfo_t;
typedef unsigned long long sigset_t;
typedef struct ucontext ucontext_t;
typedef struct mcontext mcontext_t;

struct sigaction {
  union {
    void (*sa_handler)(int);
    void (*sa_sigaction)(int, siginfo_t *, void *);
  };
  unsigned long sa_flags;
  void (*sa_restorer)(void);
  sigset_t sa_mask;
};

#endif // _BITS_SIGNAL_H
