#define _GNU_SOURCE

#include <assert.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>

#if defined(__linux__)
#include <linux/futex.h>
#include <sys/syscall.h> /* Definition of SYS_* constants */
#elif defined(__redox__)
extern size_t redox_futex_wait_v0(uint32_t* word, uint32_t val, const struct timespec* timeout);
extern size_t redox_futex_wake_v0(uint32_t* word, uint32_t max_wake);
#else
#error "Unsupported platform"
#endif

ssize_t futex_wait(volatile uint32_t* word, uint32_t val, const struct timespec* timeout)
{
#if defined(__linux__)
    return syscall(SYS_futex, (void*)word, FUTEX_WAIT, val, timeout, NULL, 0);
#elif defined(__redox__)
    return (ssize_t)redox_futex_wait_v0((uint32_t*)word, val, timeout);
#else
#error "Unsupported platform"
#endif
}

ssize_t futex_wake(volatile uint32_t* word, uint32_t max_wake)
{
#if defined(__linux__)
    return syscall(SYS_futex, (void*)word, FUTEX_WAKE, max_wake, NULL, NULL, 0);
#elif defined(__redox__)
    return (ssize_t)redox_futex_wake_v0((uint32_t*)word, max_wake);
#else
#error "Unsupported platform"
#endif
}

#define FUTEX_WORD_LOCKED 0
#define FUTEX_WORD_UNLOCKED 1

#define SHM_NAME "/my-shm-futex"

void test_shm_futex(void)
{
    int shm_fd = shm_open(SHM_NAME, O_CREAT | O_EXCL | O_RDWR, 0600);
    assert(shm_fd != -1);
    assert(ftruncate(shm_fd, sizeof(uint32_t)) != -1);

    volatile uint32_t* w1 = mmap(NULL, sizeof(uint32_t), PROT_READ | PROT_WRITE, MAP_SHARED, shm_fd, 0);
    assert(w1 != MAP_FAILED);

    *w1 = FUTEX_WORD_LOCKED;

    if (!fork()) {
        // Child
        volatile uint32_t* w2 = mmap(NULL, sizeof(uint32_t), PROT_READ | PROT_WRITE, MAP_SHARED, shm_fd, 0);
        assert(w2 != MAP_FAILED);

        // Wait for the parent to block on `futex_wait`.
        sleep(1);

        *w2 = FUTEX_WORD_UNLOCKED;
        futex_wake(w2, 1);
        exit(EXIT_SUCCESS);
    }

    // Parent
    futex_wait(w1, FUTEX_WORD_LOCKED, NULL);
    assert(*w1 == FUTEX_WORD_UNLOCKED);

    // Cleanup
    close(shm_fd);
    shm_unlink(SHM_NAME);
}

void* waiter_thr(void* arg)
{
    // The word is currently mapped to THE zero page and will be CoWed after we
    // enter `futex_wait`. Since the physical address is currently used as
    // futex key, this would block indefinitely.
    futex_wait((volatile uint32_t*)arg, 0, NULL);
    return NULL;
}

void test_private_futex(void)
{
    volatile uint32_t* word = mmap(NULL, sizeof(uint32_t), PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
    assert(word != MAP_FAILED);
    // Since word has not been written to yet, it should be mapped to THE zero page.

    pthread_t waiter;
    pthread_create(&waiter, NULL, waiter_thr, (void*)word);

    // Wait for `waiter_thr` to block on `futex_wait`.
    sleep(1);

    // Would trigger CoW.
    *word = 1;
    // The physical address of the futex word is now different.
    futex_wake(word, 1);

    pthread_join(waiter, NULL);
}

int main(void)
{
    test_private_futex();
    printf("OK private\n");
    test_shm_futex();
    printf("OK shared\n");
    return EXIT_SUCCESS;
}
