#include <assert.h>
#include <errno.h>
#include <pthread.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "common.h"

#define STACKSIZE 128 * 1024

struct retval {
  uintptr_t some_stack_pointer;
};
struct arg {
  int status;
};

void *routine(void *arg_raw) {
  struct arg *arg = arg_raw;

  struct retval *retval = malloc(sizeof(struct retval));

  if (retval == NULL) {
    arg->status = ENOMEM;
    return NULL;
  }

  char some_array[256] = { 0 };
  retval->some_stack_pointer = black_box_uintptr_t((uintptr_t)some_array);

  return retval;
}

int main(void) {
  int status;
  static char stack[STACKSIZE];

  pthread_attr_t attr;
  if ((status = pthread_attr_init(&attr)) != 0) {
    return fail(status, "attr init");
  }
  if ((status = pthread_attr_setstack(&attr, stack, STACKSIZE)) != 0) {
    return fail(status, "attr setstack");
  }

  void *stack_again;
  size_t stacksize_again;

  if ((status = pthread_attr_getstack(&attr, &stack_again, &stacksize_again)) != 0) {
    return fail(status, "attr getstack");
  }

  assert(stack_again == stack);
  assert(stacksize_again == STACKSIZE);

  pthread_t thread;

  if ((status = pthread_create(&thread, &attr, routine, NULL)) != 0) {
    return fail(status, "pthread create");
  }

  if ((status = pthread_attr_destroy(&attr)) != 0) {
    return fail(status, "attr destroy");
  }

  void *retval_raw;

  if ((status = pthread_join(thread, &retval_raw)) != 0) {
    return fail(status, "pthread join");
  }

  struct retval *retval = retval_raw;

  assert(retval->some_stack_pointer >= (uintptr_t)stack);
  assert(retval->some_stack_pointer < ((uintptr_t)stack) + STACKSIZE);

  free(retval);

  return EXIT_SUCCESS;
}
