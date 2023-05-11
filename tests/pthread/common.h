#ifndef _COMMON_H
#define _COMMON_H

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int fail(int status, const char *reason) {
  fprintf(stderr, "%s failed: %s\n", reason, strerror(status));

  return EXIT_FAILURE;
}

uintptr_t black_box_uintptr_t(uintptr_t arg) {
  // Rust implements this by putting the value behind a volatile pointer and then simply loading it.

  uintptr_t arg_slot = arg;
  volatile uintptr_t *ptr = &arg_slot;

  return *ptr;
}

#endif // _COMMON_H
