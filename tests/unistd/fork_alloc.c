#include <assert.h>
#include <err.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <sys/wait.h>

#define SIZE 32

static void alloc_at_fork(void) {
    volatile void* buf = malloc(1000000);
    free((void*) buf);
}

static void alloc_after_fork(void) {
    const char msg[] = "The cake is a lie!";
    char* before_fork = malloc((sizeof(char)) * SIZE);
    assert(before_fork);

    pid_t child = fork();
    if (child == -1) {
        err(EXIT_FAILURE, "Forking after alloc");
    } else if (child == 0) {
        char* after_fork = malloc((sizeof(char)) * SIZE);        
        assert(after_fork);
        // Write to both to check that they're valid; no segfault
        // static_assert(SIZE > sizeof(msg), "Buf is too small");
        memcpy(before_fork, msg, sizeof(msg));
        memcpy(after_fork, msg, sizeof(msg));

        free(before_fork);
        free(after_fork);
        exit(EXIT_SUCCESS);
    } else {
        char* after_fork = malloc((sizeof(char)) * SIZE);        
        assert(after_fork);

        // As above
        // static_assert(SIZE > sizeof(msg), "Buf is too small");
        memcpy(before_fork, msg, sizeof(msg));
        memcpy(after_fork, msg, sizeof(msg));
        free(after_fork);

        int status = 0;
        if (waitpid(child, &status, 0) == -1) {
            err(EXIT_FAILURE, "Waiting for spawned child to exit");
        }

        if (!WIFEXITED(status)) {
            errx(EXIT_FAILURE, "Child process failed");
        }
    }

    free(before_fork);
}

int main(void) {
    int result = pthread_atfork(alloc_at_fork, alloc_at_fork, alloc_at_fork);
    if (result == -1) {
        err(EXIT_FAILURE, "Adding alloc_at_fork hooks");
    }

    alloc_after_fork();

    return EXIT_SUCCESS;
}
