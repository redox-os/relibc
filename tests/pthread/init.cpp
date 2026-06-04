// this file is both C and CPP to check NULL and thread_local on C++ environment

#include <stdio.h>
#include <pthread.h>
#include <stddef.h>
#include <assert.h>
#include <threads.h>

static pthread_cond_t   global_cond   = PTHREAD_COND_INITIALIZER;
static pthread_mutex_t  global_mutex  = PTHREAD_MUTEX_INITIALIZER;
static pthread_rwlock_t global_rwlock = PTHREAD_RWLOCK_INITIALIZER;
// TODO: POSIX mandates *_INITIALIZER as const initializer but not PTHREAD_ONCE_INIT
//       But glibc and other platform support it. Ours cannot in C due to union type
#ifdef __cplusplus
static pthread_once_t   global_once   = PTHREAD_ONCE_INIT;
#else
static pthread_once_t   global_once   = {0};
#endif

static void once_callback(void) {
    printf("once\n");
}

static thread_local int tls_counter = 100; 

void* thread_func(void* args) {
    (void)args;
    assert(tls_counter == 100);
    tls_counter = 200;
    assert(tls_counter == 200);
    return NULL;
}

int main(void) {
    assert(pthread_mutex_lock(&global_mutex) == 0);
    assert(pthread_mutex_unlock(&global_mutex) == 0);
    assert(pthread_rwlock_wrlock(&global_rwlock) == 0);
    assert(pthread_rwlock_unlock(&global_rwlock) == 0);
    assert(pthread_once(&global_once, once_callback) == 0);
    assert(pthread_cond_signal(&global_cond) == 0);

    pthread_t thread_id;
    int create_res = pthread_create(&thread_id, NULL, thread_func, NULL);
    assert(create_res == 0);
    pthread_mutex_t dynamic_mutex;
    int mutex_res = pthread_mutex_init(&dynamic_mutex, NULL);
    assert(mutex_res == 0);
    assert(pthread_join(thread_id, NULL) == 0);
    assert(tls_counter == 100);

    return 0;
}
