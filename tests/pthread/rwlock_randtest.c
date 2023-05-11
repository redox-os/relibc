#include "../test_helpers.h"
#include "common.h"

#include <errno.h>
#include <pthread.h>
#include <stdlib.h>

// Same test logic as frob in rustc/library/std/sync/rwlock/tests.rs

#define N 10
//#define M 1000
#define M 100000

struct arg {
  pthread_rwlock_t *rwlock;
};

void *routine(void *arg_raw) {
  struct arg *arg = arg_raw;
  int status;

  for (uint64_t i = 0; i < M; i++) {
    if (random_bool()) {
      status = pthread_rwlock_wrlock(arg->rwlock);
      ERROR_IF(pthread_rwlock_wrlock, status, != 0);
    } else {
      status = pthread_rwlock_rdlock(arg->rwlock);
      ERROR_IF(pthread_rwlock_rdlock, status, != 0);
    }
    status = pthread_rwlock_unlock(arg->rwlock);
    ERROR_IF(pthread_rwlock_unlock, status, != 0);
  }

  return NULL;
}

int main(void) {
  int status;

  pthread_rwlock_t rwlock;

  status = pthread_rwlock_init(&rwlock, NULL);
  ERROR_IF(pthread_rwlock_init, status, != 0);

  pthread_t threads[N];
  struct arg args[N];

  for (size_t i = 0; i < N; i++) {
    args[i] = (struct arg){ .rwlock = &rwlock };

    status = pthread_create(&threads[i], NULL, routine, &args[i]);
    ERROR_IF(pthread_create, status, != 0);
  }

  for (size_t i = 0; i < N; i++) {
    status = pthread_join(threads[i], NULL);
    ERROR_IF(pthread_join, status, != 0);
  }

  status = pthread_rwlock_destroy(&rwlock);
  ERROR_IF(pthread_rwlock_destroy, status, != 0);
  
  return 0;
}
