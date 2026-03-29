#include "../test_helpers.h"
#include "common.h"

#include <assert.h>
#include <pthread.h>
#include <errno.h>
#include <time.h>

// Adapted from test_rwlock_try_write in rustc/library/std/sync/rwlock/tests.rs

int main(void) {
  int status;
  pthread_rwlock_t rwlock = PTHREAD_RWLOCK_INITIALIZER;

  pthread_rwlockattr_t attr;
  status = pthread_rwlockattr_init(&attr);
  ERROR_IF(pthread_rwlockattr_init, status, != 0);

  // Call setpshared twice to check both constants work.
  status = pthread_rwlockattr_setpshared(&attr, PTHREAD_PROCESS_SHARED);
  ERROR_IF(pthread_rwlockattr_setpshared, status, != 0);
  status = pthread_rwlockattr_setpshared(&attr, PTHREAD_PROCESS_PRIVATE);
  ERROR_IF(pthread_rwlockattr_setpshared, status, != 0);

  status = pthread_rwlock_init(&rwlock, &attr);
  ERROR_IF(pthread_rwlock_init, status, != 0);

  status = pthread_rwlockattr_destroy(&attr);
  ERROR_IF(pthread_rwlockattr_destroy, status, != 0);

  status = pthread_rwlock_rdlock(&rwlock);
  ERROR_IF(pthread_rwlock_rdlock, status, != 0);

  status = pthread_rwlock_trywrlock(&rwlock);
  UNEXP_IF(pthread_rwlock_trywrlock, status, != EBUSY);

  struct timespec ts;
  clock_gettime(CLOCK_REALTIME, &ts);
  ts.tv_nsec += 200 * 1000000;
  status = pthread_rwlock_timedwrlock(&rwlock, &ts);
  UNEXP_IF(pthread_rwlock_timedwrlock, status, != ETIMEDOUT);

  status = pthread_rwlock_unlock(&rwlock);
  ERROR_IF(pthread_rwlock_unlock, status, != 0);

  status = pthread_rwlock_trywrlock(&rwlock);
  ERROR_IF(pthread_rwlock_rdlock, status, != 0);

  clock_gettime(CLOCK_REALTIME, &ts);
  ts.tv_nsec += 200 * 1000000;
  status = pthread_rwlock_timedrdlock(&rwlock, &ts);
  UNEXP_IF(pthread_rwlock_timedrdlock, status, != ETIMEDOUT);

  status = pthread_rwlock_destroy(&rwlock);
  ERROR_IF(pthread_rwlock_destroy, status, != 0);

  return 0;
}
