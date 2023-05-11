#include <pthread.h>
#include <stdbool.h>
#include <stdlib.h>

#include "../test_helpers.h"

#define N 10
#define M 10000

struct arg {
  pthread_mutex_t *mutex;
  unsigned *protected;
};

void *routine(void *arg_raw) {
  struct arg *arg = arg_raw;
  int status;

  unsigned depth = 0;
  unsigned i = 0;

  while (i < M) {
    // Bad random distribution, but should work.
    bool lock_again = (depth == 0) || (random_bool() && random_bool());

    if (lock_again) {
      status = pthread_mutex_lock(arg->mutex);
      ERROR_IF(pthread_mutex_lock, status, != 0);

      depth += 1;
    } else {
      status = pthread_mutex_unlock(arg->mutex);
      ERROR_IF(pthread_mutex_unlock, status, != 0);

      depth -= 1;
    }
    if (depth == 0) {
      continue;
    }

    unsigned value = *arg->protected;
    *arg->protected = value + 1;

    i += 1;
  }
  while (depth > 0) {
    status = pthread_mutex_unlock(arg->mutex);
    ERROR_IF(pthread_mutex_unlock, status, != 0);

    depth--;
  }

  return NULL;
}

int main(void) {
  int status;
  pthread_mutex_t mutex;
  pthread_mutexattr_t attr;

  status = pthread_mutexattr_init(&attr);
  ERROR_IF(pthread_mutexattr_init, status, != 0);

  status = pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
  ERROR_IF(pthread_mutexattr_settype, status, != 0);

  status = pthread_mutex_init(&mutex, &attr);
  ERROR_IF(pthread_mutex_init, status, != 0);

  status = pthread_mutexattr_destroy(&attr);
  ERROR_IF(pthread_mutexattr_destroy, status, != 0);

  status = pthread_mutex_lock(&mutex);
  ERROR_IF(pthread_mutex_lock, status, != 0);

  status = pthread_mutex_trylock(&mutex);
  ERROR_IF(pthread_mutex_trylock, status, != 0);

  status = pthread_mutex_unlock(&mutex);
  ERROR_IF(pthread_mutex_unlock, status, != 0);

  // Still locked with count = 1.

  pthread_t threads[N];
  struct arg args[N];
  unsigned protected = 0;

  for (size_t i = 0; i < N; i++) {
    args[i] = (struct arg){ .mutex = &mutex, .protected = &protected };
    status = pthread_create(&threads[i], NULL, routine, &args[i]);
    ERROR_IF(pthread_create, status, != 0);
  }

  protected = 1;

  status = pthread_mutex_unlock(&mutex);
  ERROR_IF(pthread_mutex_unlock, status, != 0);

  for (size_t i = 0; i < N; i++) {
    status = pthread_join(threads[i], NULL);
    ERROR_IF(pthread_join, status, != 0);
  }

  status = pthread_mutex_destroy(&mutex);
  ERROR_IF(pthread_mutex_destroy, status, != 0);

  return 0;
}
