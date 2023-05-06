#include <assert.h>
#include <pthread.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "common.h"
#include "../test_helpers.h"

_Thread_local size_t this_thread_count = 0;

#define COUNT 1024
#define THREADS 4

static void constructor(void) {
  this_thread_count++;
}

struct arg {
  pthread_barrier_t *barrier;
  pthread_once_t *onces;
  size_t count;
  size_t index;
};

void *routine(void *arg_raw) {
  int status;
  struct arg *arg = arg_raw;

  if (arg->index == THREADS - 1) {
    printf("main thread at %zu\n", arg->index);
  } else {
    printf("spawned %zu\n", arg->index);
  }

  status = pthread_barrier_wait(arg->barrier);

  printf("waited %zu leader=%s\n", arg->index, (status == PTHREAD_BARRIER_SERIAL_THREAD) ? "true" : "false");

  if (status != PTHREAD_BARRIER_SERIAL_THREAD) {
    ERROR_IF(pthread_barrier_wait, status, != 0);
    return NULL;
  }

  for (size_t i = 0; i < COUNT; i++) {
    status = pthread_once(&arg->onces[i], constructor);
    ERROR_IF(pthread_once, status, != 0);
  }

  arg->count = this_thread_count;

  return NULL;
}

int main(void) {
  // TODO: Better test to simulate contention?

  int status;

  pthread_once_t onces[COUNT];

  for (size_t i = 0; i < COUNT; i++) {
    onces[i] = PTHREAD_ONCE_INIT;
  }

  pthread_barrier_t barrier;

  printf("Barrier at %p, onces at %p\n", &barrier, onces);

  status = pthread_barrier_init(&barrier, NULL, THREADS);
  ERROR_IF(pthread_barrier_init, status, != 0);

  pthread_t threads[THREADS];
  struct arg args[THREADS];

  for (size_t i = 0; i < THREADS; i++) {
    args[i] = (struct arg){ .barrier = &barrier, .onces = onces, .count = 0, .index = i };
    printf("spawning %zu\n", i);

    status = pthread_create(&threads[i], NULL, routine, &args[i]);
    ERROR_IF(pthread_create, status, != 0);
  }

  size_t total_count = 0;

  for (size_t i = 0; i < THREADS; i++) {
    status = pthread_join(threads[i], NULL);
    ERROR_IF(pthread_join, status, != 0);

    total_count += args[i].count;
  }

  status = pthread_barrier_destroy(&barrier);
  ERROR_IF(pthread_barrier_destroy, status, != 0);

  assert(total_count == COUNT);

  return EXIT_SUCCESS;
}
