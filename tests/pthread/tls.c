#include <assert.h>
// PTHREAD_KEYS_MAX, PTHREAD_DESTRUCTOR_ITERATIONS
#include <limits.h>
#include <pthread.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

pthread_key_t key_a, key_b, key_c, key_d, key_e;
size_t total_destructor_runs = 0;

void drop_key_a(void *val_a) {
    assert(pthread_getspecific(key_a) == NULL && val_a == (void *)0xaaaa);
    total_destructor_runs++;

    void *val_b = pthread_getspecific(key_b);
    printf("drop_key_a(a=%p, b=%p)\n", val_a, val_b);

    if (total_destructor_runs <= 2) {
        pthread_setspecific(key_a, val_a);
        return;
    }

    if (val_b != NULL) {
        assert(val_b == (void *)0xbbbb);
    } else {
        pthread_setspecific(key_b, (void *)0xbbbb);
    }
}

void drop_key_b(void *val_b) {
    assert(pthread_getspecific(key_b) == NULL && val_b == (void *)0xbbbb);
    total_destructor_runs++;

    void *val_a = pthread_getspecific(key_a);
    printf("drop_key_b(a=%p, b=%p)\n", val_a, val_b);

    if (total_destructor_runs <= 2) {
        pthread_setspecific(key_b, val_b);
        return;
    }

    if (val_a != NULL) {
        assert(val_a == (void *)0xaaaa);
    } else {
        pthread_setspecific(key_a, (void *)0xaaaa);
    }
}

void drop_unreachable(void *val_c) {
    // Should never be called.
    (void)val_c;
    assert(false);
}

void drop_key_d(void *val_d) {
    assert(pthread_getspecific(key_d) == NULL && val_d == (void *)0xdddd);
    // [`pthread_key_{create,setspecific,getspecific,delete}`] can still be used
    // inside the destructor itself.
    printf("drop_key_d(d=%p)\n", val_d);
    pthread_key_t key_g;
    assert(!pthread_key_create(&key_g, drop_unreachable));
    assert(!pthread_setspecific(key_g, &key_g));
    assert(pthread_getspecific(key_g) == &key_g);
    assert(!pthread_key_delete(key_g));
}

void *test_tls_destructors(void *arg) {
    (void)arg;
    assert(!pthread_key_create(&key_a, drop_key_a));
    assert(!pthread_key_create(&key_b, drop_key_b));
    assert(!pthread_key_create(&key_c, drop_unreachable));
    assert(!pthread_key_create(&key_d, drop_key_d));
    assert(!pthread_key_create(&key_e, drop_unreachable));
    assert(!pthread_setspecific(key_a, (void *)0xaaaa));
    assert(!pthread_setspecific(key_b, (void *)0xbbbb));
    assert(!pthread_setspecific(key_c, (void *)0xcccc));
    assert(!pthread_setspecific(key_d, (void *)0xdddd));
    assert(!pthread_key_delete(key_c));
    return NULL;
}

int main(void) {
    pthread_key_t key_f;
    assert(!pthread_key_create(&key_f, NULL));
    assert(!pthread_setspecific(key_f, &key_f));
    assert(pthread_getspecific(key_f) == &key_f);
    assert(!pthread_key_delete(key_f));

    pthread_t t1;
    assert(!pthread_create(&t1, NULL, test_tls_destructors, NULL));
    assert(!pthread_join(t1, NULL));
    printf("total_destructor_iterations: %zu\n", total_destructor_runs);
    // There are two destructors (i.e., `drop_key_{a, b}`), so the total number
    // of iterations will be `2 * PTHREAD_DESTRUCTOR_ITERATIONS`.
    assert((total_destructor_runs / 2) == PTHREAD_DESTRUCTOR_ITERATIONS);

    return EXIT_SUCCESS;
}
