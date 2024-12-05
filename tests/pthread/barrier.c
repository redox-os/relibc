#include "../test_helpers.h"
#include "common.h"

#include <assert.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdbool.h>

// Same test logic as test_barrier in rustc/library/std/sync/barrier/tests.rs

#define N 10

struct arg {
  pthread_barrier_t *barrier;
  volatile _Atomic(unsigned) *count;
  bool is_leader;
};

void *routine(void *arg_raw) {
  struct arg *arg = arg_raw;

  int status = pthread_barrier_wait(arg->barrier);

  arg->is_leader = status == PTHREAD_BARRIER_SERIAL_THREAD;

  if (!arg->is_leader)
    ERROR_IF(pthread_barrier_wait, status, != 0);

  // We can now modify the counter.
  atomic_fetch_add_explicit(arg->count, 1, memory_order_relaxed);

  return NULL;
}

int main(void) {
  int status;

  pthread_barrier_t barrier;

  pthread_barrierattr_t attr;
  status = pthread_barrierattr_init(&attr);
  ERROR_IF(pthread_barrierattr_init, status, != 0);

  int pshared;

  //
  // BARRIER ATTR
  //

  status = pthread_barrierattr_getpshared(&attr, &pshared);

  // PTHREAD_PROCESS_PRIVATE is default according to POSIX.
  assert(pshared == PTHREAD_PROCESS_PRIVATE);

  ERROR_IF(pthread_barrierattr_getpshared, status, != 0);

  status = pthread_barrierattr_setpshared(&attr, PTHREAD_PROCESS_SHARED);
  ERROR_IF(pthread_barrierattr_setpshared, status, != 0);

  status = pthread_barrierattr_getpshared(&attr, &pshared);
  assert(pshared == PTHREAD_PROCESS_SHARED);
  ERROR_IF(pthread_barrierattr_getpshared, status, != 0);

  status = pthread_barrierattr_setpshared(&attr, PTHREAD_PROCESS_PRIVATE);
  ERROR_IF(pthread_barrierattr_setpshared, status, != 0);

  status = pthread_barrierattr_getpshared(&attr, &pshared);
  assert(pshared == PTHREAD_PROCESS_PRIVATE);
  ERROR_IF(pthread_barrierattr_getpshared, status, != 0);

  //
  // BARRIER
  //

  status = pthread_barrier_init(&barrier, &attr, N);
  ERROR_IF(pthread_barrier_init, status, != 0);

  status = pthread_barrierattr_destroy(&attr);
  ERROR_IF(pthread_barrierattr_destroy, status, != 0);

  //
  // CREATE THREAD
  //

  pthread_t threads[N - 1];
  struct arg args[N - 1];
  _Atomic(unsigned) count = false;

  for (size_t i = 0; i < N - 1; i++) {
    args[i] = (struct arg){ .count = &count, .barrier = &barrier, .is_leader = false };
    status = pthread_create(&threads[i], NULL, routine, &args[i]);
    ERROR_IF(pthread_create, status, != 0);
  }

  // Must not be set before having waited for the barrier. This is part of the
  // test. Normally spawned threads run before the scheduler returns to the
  // parent thread, so it should at least partially verify that barriers work.
  unsigned value = atomic_load_explicit(&count, memory_order_relaxed);

  UNEXP_IF(count_before_barrier_wait, value, > 0);

  status = pthread_barrier_wait(&barrier);

  bool leader_found = status == PTHREAD_BARRIER_SERIAL_THREAD;

  if (!leader_found) {
    ERROR_IF(pthread_barrier_wait, status, != 0);
  }

  for (size_t i = 0; i < N - 1; i++) {
    status = pthread_join(threads[i], NULL);
    ERROR_IF(pthread_join, status, != 0);

    // SAFETY: pthread_create and pthread_join are Acquire-Release
    leader_found |= args[i].is_leader;
  }

  assert(leader_found);

  status = pthread_barrier_destroy(&barrier);
  ERROR_IF(pthread_barrier_destroy, status, != 0);

  return 0;
}
