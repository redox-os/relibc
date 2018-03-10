#ifndef _BITS_SIGNAL_H
#define _BITS_SIGNAL_H

typedef struct sigaction {
  void (*sa_handler)(uintptr_t);
  sigset_t sa_mask;
  uintptr_t sa_flags;
};

#endif
