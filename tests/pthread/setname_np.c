#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* Prototype for thread name functions */
extern int pthread_setname_np(pthread_t thread, const char *name);
extern int pthread_getname_np(pthread_t thread, char *name, size_t len);

int main(void) {
    printf("Testing pthread_setname_np\n");
    
    pthread_t self = pthread_self();
    printf("Self thread pointer: %p\n", (void*)self);
    
    /* Test 1: Basic name set and get */
    const char *test_name = "main-thread";
    printf("\nTest 1: Setting name to '%s'\n", test_name);
    
    int status = pthread_setname_np(self, test_name);
    printf("pthread_setname_np returned: %d\n", status);
    
    if (status == 0) {
        char buffer[64];
        
        /* Get name for current thread */
        status = pthread_getname_np(self, buffer, sizeof(buffer));
        printf("pthread_getname_np returned: %d\n", status);
        
        if (status == 0) {
            printf("Thread name retrieved: '%s'\n", buffer);
            
            /* Verify the name matches what we set */
            if (strcmp(buffer, test_name) == 0) {
                printf("SUCCESS: Retrieved name matches set name\n");
            } else {
                printf("ERROR: Retrieved name '%s' does not match set name '%s'\n", 
                       buffer, test_name);
                return 1;
            }
        }
    }
    
    /* Test 2: Set and retrieve a different name */
    const char *test_name2 = "renamed-thread";
    printf("\nTest 2: Changing name to '%s'\n", test_name2);
    
    status = pthread_setname_np(self, test_name2);
    printf("pthread_setname_np returned: %d\n", status);
    
    if (status == 0) {
        char buffer[64];
        
        /* Get name for current thread */
        status = pthread_getname_np(self, buffer, sizeof(buffer));
        printf("pthread_getname_np returned: %d\n", status);
        
        if (status == 0) {
            printf("Thread name retrieved: '%s'\n", buffer);
            
            /* Verify the name matches what we set */
            if (strcmp(buffer, test_name2) == 0) {
                printf("SUCCESS: Retrieved name matches set name\n");
            } else {
                printf("ERROR: Retrieved name '%s' does not match set name '%s'\n", 
                       buffer, test_name2);
                return 1;
            }
        }
    }
    
    /* Test 3: Verify name truncation when buffer is too small */
    printf("\nTest 3: Testing buffer truncation\n");
    
    char small_buffer[5]; /* Only room for 4 chars + null terminator */
    status = pthread_getname_np(self, small_buffer, sizeof(small_buffer));
    printf("pthread_getname_np with small buffer returned: %d\n", status);
    
    /* On some systems like glibc, truncation returns success (0)
       On others, it might return ERANGE (34) to indicate truncation */
    if (status == 0) {
        printf("Truncated name: '%s'\n", small_buffer);
        
        /* Check that small buffer contains the truncated name */
        if (strlen(small_buffer) == 4 && 
            strncmp(small_buffer, test_name2, 4) == 0) {
            printf("SUCCESS: Name was correctly truncated (returned 0)\n");
        } else {
            printf("ERROR: Expected truncated name to be '%.4s'\n", test_name2);
            return 1;
        }
    } else if (status == 34) { /* ERANGE */
        printf("SUCCESS: Truncation correctly reported with ERANGE\n");
    } else {
        printf("ERROR: Unexpected error code: %d\n", status);
        return 1;
    }
    
    printf("\nAll tests completed successfully\n");
    return 0;
}