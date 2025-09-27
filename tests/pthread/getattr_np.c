#include <assert.h>
#include <errno.h>
#include <pthread.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>  /* For usleep() */

// Disable specific warnings for this test file
#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

#include "common.h"

/* Prototype for pthread_getattr_np since it might not be in the headers yet */
extern int pthread_getattr_np(pthread_t thread, pthread_attr_t *attr);

#define CUSTOM_STACKSIZE (256 * 1024)
static char custom_stack[CUSTOM_STACKSIZE];

void *thread_with_default_attrs(void *arg) {
    // Just return the arg
    return arg;
}

void *thread_with_custom_stack(void *_arg) {
    // Mark that we're using a custom stack by storing a value on the stack
    char buffer[128] = {42};
    
    // Use a pointer to our stack buffer as proof we're in the custom stack
    uintptr_t _stack_ptr = black_box_uintptr_t((uintptr_t)buffer);
    
    // Return a success indicator
    return (void*)1;
}

void *detached_thread(void *_arg) {
    // This is a detached thread that just returns
    return NULL;
}

int test_default_attributes() {
    int status;
    pthread_t thread;
    void *retval;
    
    printf("Testing pthread_getattr_np with default attributes\n");
    
    // Create a thread with default attributes
    if ((status = pthread_create(&thread, NULL, thread_with_default_attrs, (void*)0x12345678)) != 0) {
        return fail(status, "pthread_create with default attributes");
    }
    
    // Get the attributes of the created thread
    pthread_attr_t attr;
    if ((status = pthread_getattr_np(thread, &attr)) != 0) {
        return fail(status, "pthread_getattr_np");
    }
    
    // Check detachstate (should be joinable by default)
    int detachstate;
    if ((status = pthread_attr_getdetachstate(&attr, &detachstate)) != 0) {
        return fail(status, "pthread_attr_getdetachstate");
    }
    assert(detachstate == PTHREAD_CREATE_JOINABLE);
    printf("Default detachstate: %s\n", detachstate == PTHREAD_CREATE_JOINABLE ? "JOINABLE" : "DETACHED");
    
    // Check stack information
    void *stackaddr;
    size_t stacksize;
    if ((status = pthread_attr_getstack(&attr, &stackaddr, &stacksize)) != 0) {
        return fail(status, "pthread_attr_getstack");
    }
    
    printf("Thread stack address: %p, size: %zu bytes\n", stackaddr, stacksize);
    assert(stackaddr != NULL);
    assert(stacksize > 0);
    
    // Join the thread
    if ((status = pthread_join(thread, &retval)) != 0) {
        return fail(status, "pthread_join");
    }
    
    // Check if we got the expected return value
    assert(retval == (void*)0x12345678);
    
    // Clean up
    if ((status = pthread_attr_destroy(&attr)) != 0) {
        return fail(status, "pthread_attr_destroy");
    }
    
    return 0;
}

int test_custom_stack() {
    int status;
    pthread_t thread;
    void *retval;
    
    printf("Testing pthread_getattr_np with custom stack\n");
    
    // Create attributes with custom stack
    pthread_attr_t create_attr;
    if ((status = pthread_attr_init(&create_attr)) != 0) {
        return fail(status, "pthread_attr_init");
    }
    
    if ((status = pthread_attr_setstack(&create_attr, custom_stack, CUSTOM_STACKSIZE)) != 0) {
        return fail(status, "pthread_attr_setstack");
    }
    
    // Create a thread with custom stack
    if ((status = pthread_create(&thread, &create_attr, thread_with_custom_stack, NULL)) != 0) {
        return fail(status, "pthread_create with custom stack");
    }
    
    // Destroy the attr used for creation
    if ((status = pthread_attr_destroy(&create_attr)) != 0) {
        return fail(status, "pthread_attr_destroy");
    }
    
    // Get the attributes of the created thread
    pthread_attr_t attr;
    if ((status = pthread_getattr_np(thread, &attr)) != 0) {
        return fail(status, "pthread_getattr_np");
    }
    
    // Check stack information
    void *stackaddr;
    size_t stacksize;
    if ((status = pthread_attr_getstack(&attr, &stackaddr, &stacksize)) != 0) {
        return fail(status, "pthread_attr_getstack");
    }
    
    printf("Custom thread stack address: %p, size: %zu bytes\n", stackaddr, stacksize);
    
    // Verify it matches what we set
    assert(stackaddr == custom_stack);
    assert(stacksize == CUSTOM_STACKSIZE);
    
    // Join the thread
    if ((status = pthread_join(thread, &retval)) != 0) {
        return fail(status, "pthread_join");
    }
    
    // Check if we got the expected return value
    assert(retval == (void*)1);
    
    // Clean up
    if ((status = pthread_attr_destroy(&attr)) != 0) {
        return fail(status, "pthread_attr_destroy");
    }
    
    return 0;
}

int test_detached_thread() {
    int status;
    pthread_t thread;
    
    printf("Testing pthread_getattr_np with detached thread\n");
    
    // Create attributes for detached thread
    pthread_attr_t create_attr;
    if ((status = pthread_attr_init(&create_attr)) != 0) {
        return fail(status, "pthread_attr_init");
    }
    
    if ((status = pthread_attr_setdetachstate(&create_attr, PTHREAD_CREATE_DETACHED)) != 0) {
        return fail(status, "pthread_attr_setdetachstate");
    }
    
    // Create a detached thread
    if ((status = pthread_create(&thread, &create_attr, detached_thread, NULL)) != 0) {
        return fail(status, "pthread_create detached thread");
    }
    
    // Destroy the attr used for creation
    if ((status = pthread_attr_destroy(&create_attr)) != 0) {
        return fail(status, "pthread_attr_destroy");
    }
    
    // Small sleep to ensure the thread has started
    usleep(100000); // 100ms
    
    // Get the attributes of the created thread
    pthread_attr_t attr;
    if ((status = pthread_getattr_np(thread, &attr)) != 0) {
        // If the thread has already exited, this might fail
        // This is implementation-defined behavior for detached threads
        printf("pthread_getattr_np on detached thread returned %d (%s)\n", 
               status, strerror(status));
        return 0;
    }
    
    // Check detachstate
    int detachstate;
    if ((status = pthread_attr_getdetachstate(&attr, &detachstate)) != 0) {
        return fail(status, "pthread_attr_getdetachstate");
    }
    
    assert(detachstate == PTHREAD_CREATE_DETACHED);
    printf("Detached thread state: %s\n", 
           detachstate == PTHREAD_CREATE_DETACHED ? "DETACHED" : "JOINABLE");
    
    // Clean up
    if ((status = pthread_attr_destroy(&attr)) != 0) {
        return fail(status, "pthread_attr_destroy");
    }
    
    // Note: We don't join a detached thread
    
    return 0;
}

int main(void) {
    int result;
    
    // Test with default attributes
    if ((result = test_default_attributes()) != 0) {
        return result;
    }
    
    // Test with custom stack
    if ((result = test_custom_stack()) != 0) {
        return result;
    }
    
    // Test with detached thread
    if ((result = test_detached_thread()) != 0) {
        return result;
    }
    
    printf("All tests passed!\n");
    return EXIT_SUCCESS;
}
