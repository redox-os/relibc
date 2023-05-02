#include <assert.h>
#include <pthread.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "common.h"

struct once_data {
  size_t count;
};

_Thread_local struct once_data once_data = {0};

#define COUNT 1024
#define THREADS 4

void constructor() {
  once_data.count += 1;
}

struct arg {
  pthread_barrier_t *barrier;
  pthread_once_t *onces;
  int status;
  size_t count;
  size_t index;
};

void *routine(void *arg_raw) {
  struct arg *arg = arg_raw;

  if (arg->index == THREADS - 1) {
    printf("main thread at %zu\n", arg->index);
  } else {
    printf("spawned %zu\n", arg->index);
  }

  int wait_status = pthread_barrier_wait(arg->barrier);

  printf("waited %zu leader=%s\n", arg->index, (wait_status == PTHREAD_BARRIER_SERIAL_THREAD) ? "true" : "false");

  if (wait_status != PTHREAD_BARRIER_SERIAL_THREAD && wait_status != 0) {
    return NULL;
  }

  for (size_t i = 0; i < COUNT; i++) {
    if ((arg->status = pthread_once(&arg->onces[i], constructor)) != 0) {
      return NULL;
    }
  }

  arg->count = once_data.count;

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

  if ((status = pthread_barrier_init(&barrier, NULL, THREADS)) != 0) {
    return fail(status, "barrier init");
  }

  pthread_t threads[THREADS];
  struct arg args[THREADS];

  threads[0] = pthread_self();

  for (size_t i = 0; i < THREADS; i++) {
    args[i] = (struct arg){ .barrier = &barrier, .onces = onces, .status = 0, .count = 0, .index = i };
    printf("spawning %zu\n", i);

    if (i == 0) {
      continue;
    }

    if ((status = pthread_create(&threads[i], NULL, routine, &args[i])) != 0) {
      return fail(status, "thread create");
    }
  }

  routine(&args[0]);

  size_t total_count = 0;

  for (size_t i = 0; i < THREADS; i++) {
    if (i != 0) {
      if ((status = pthread_join(threads[i], NULL)) != 0) {
        return fail(status, "join");
      }
    }
    if (args[i].status != 0) {
      return fail(args[i].status, "thread");
    }
    total_count += args[i].count;
  }

  assert(total_count == COUNT);

  return EXIT_SUCCESS;
}
