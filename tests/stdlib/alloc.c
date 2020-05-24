#include <malloc.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <stddef.h> /* for size_t */
#include <stdint.h> /* for SIZE_MAX */
#include <string.h> /* for strerror() */
#include <unistd.h> /* for sysconf() */

#include "test_helpers.h"

/* For regular allocations that should succeed without particular
 * alignment requirements. */
void test_non_null(void *ptr, int error_val) {
    if (ptr != NULL) {
        // Constant output for successful case
        printf("pointer: (not NULL), ");
    }
    else {
        printf("pointer: %p, ", ptr);
    }
    printf("error value: %d = %s\n",
        error_val, strerror(error_val));
}

/* For testing functions that should return pointers with a particular
 * alignment (successful case). */
void test_valid_aligned(void *ptr, size_t alignment, int error_val) {
    /* Cast to uintptr_t to allow taking modulo of address. The
     * uintptr_t type is guaranteed to be able to hold any valid object
     * address. */
    uintptr_t ptr_alignment_rem = (uintptr_t)ptr % (uintptr_t)alignment;
    
    if (ptr != NULL && ptr_alignment_rem == 0) {
        // Constant output for successful case
        printf("pointer: (alignment OK), ");
    }
    else {
        printf("pointer: %p, ", ptr);
    }
    printf("error value: %d = %s\n",
        error_val, strerror(error_val));
}

/* For testing functions that should return pointers with a particular
 * alignment. With invalid alignment, we expect constant output (a NULL
 * pointer and EINVAL). */
void test_invalid_aligned(void *ptr, int error_val) {
    printf("pointer: %p, error value: %d = %s\n",
        ptr, error_val, strerror(error_val));
}

/* For testing size-0 allocation requests. */
void test_size_zero(void *ptr, size_t alignment, int error_val) {
    /* Facilitates checking alignment upon non-NULL return */
    uintptr_t ptr_alignment_rem = (uintptr_t)ptr % (uintptr_t)alignment;
    
    /* For allocation functions, POSIX permits returning either a NULL
     * pointer and optionally an implementation-defined error value, or
     * succeeding with a non-NULL pointer. */
    if (ptr == NULL || (ptr_alignment_rem == 0 && error_val == 0)) {
        // Constant output for successful case
        printf("(OK)\n");
    }
    else {
        printf("pointer: %p, error value: %d = %s\n",
        ptr, error_val, strerror(error_val));
    }
}

/* For cases where we expect allocation to fail, returning a NULL
 * pointer and indicating ENOMEM. */
void test_cannot_alloc(void *ptr, int error_val) {
    printf("pointer: %p, error value: %d = %s\n",
        ptr, error_val, strerror(error_val));
}

int main(void) {
    size_t sample_alloc_size = 256;
    size_t sample_realloc_size = sample_alloc_size + 1;
    
    /* ensure values are mapped to variables */
    size_t zero_size = 0;
    size_t max_size = SIZE_MAX;
    size_t page_size = (size_t)sysconf(_SC_PAGESIZE);
    size_t aligned_alloc_alignment = 128;
    size_t aligned_alloc_goodsize = 256;
    size_t aligned_alloc_badsize = 257;
    size_t nonpow2_mul_voidptr_size = 3*sizeof(void *);
    size_t pow2_mul_voidptr_size = 4*sizeof(void *);
    
    int i;
    
    errno = 0;
    char * ptr_zerosize_malloc = (char *)malloc(zero_size);
    int malloc_zerosize_errno = errno;
    printf("malloc (size 0): ");
    test_size_zero(ptr_zerosize_malloc, 1, malloc_zerosize_errno);
    free(ptr_zerosize_malloc);
    
    errno = 0;
    char * ptr_malloc = (char *)malloc(sample_alloc_size);
    int malloc_errno = errno;
    printf("malloc: ");
    test_non_null(ptr_malloc, malloc_errno);
    for(i = 0; i < sample_alloc_size; i++) {
        ptr_malloc[i] = (char)i;
    }
    free(ptr_malloc);
    
    errno = 0;
    char * ptr_malloc_maxsize = (char *)malloc(max_size);
    int malloc_maxsize_errno = errno;
    printf("malloc (SIZE_MAX): ");
    test_cannot_alloc(ptr_malloc_maxsize, malloc_maxsize_errno);
    free(ptr_malloc_maxsize);
    
    errno = 0;
    char * ptr_zerosize_calloc = (char *)calloc(zero_size, 1);
    int calloc_zerosize_errno = errno;
    printf("calloc (size 0): ");
    test_size_zero(ptr_zerosize_calloc, 1, calloc_zerosize_errno);
    free(ptr_zerosize_calloc);
    
    errno = 0;
    char * ptr_calloc = (char *)calloc(sample_alloc_size, 1);
    int calloc_errno = errno;
    printf("calloc: ");
    test_non_null(ptr_calloc, calloc_errno);
    for(i = 0; i < sample_alloc_size; i++) {
        ptr_calloc[i] = (char)i;
    }
    free(ptr_calloc);
    
    errno = 0;
    char * ptr_calloc_overflow = (char *)calloc(max_size, max_size);
    int calloc_overflow_errno = errno;
    printf("calloc (overflowing): ");
    test_cannot_alloc(ptr_calloc_overflow, calloc_overflow_errno);
    free(ptr_calloc_overflow);
    
    char * ptr_realloc_size0 = (char *)malloc(sample_alloc_size);
    errno = 0;
    ptr_realloc_size0 = (char *)realloc(ptr_realloc_size0, zero_size);
    int realloc_size0_errno = errno;
    printf("realloc (size 0): ");
    test_size_zero(ptr_realloc_size0, 1, realloc_size0_errno);
    free(ptr_realloc_size0);
    
    char * ptr_realloc = (char *)malloc(sample_alloc_size);
    errno = 0;
    ptr_realloc = (char *)realloc(ptr_realloc, sample_realloc_size);
    int realloc_errno = errno;
    printf("realloc: ");
    test_non_null(ptr_realloc, realloc_errno);
    for(i = 0; i < sample_realloc_size; i++) {
        ptr_realloc[i] = (char)i;
    }
    free(ptr_realloc);
    
    char * ptr_realloc_maxsize = (char *)malloc(sample_alloc_size);
    errno = 0;
    ptr_realloc_maxsize = (char *)realloc(ptr_realloc_maxsize, max_size);
    int realloc_maxsize_errno = errno;
    printf("realloc (SIZE_MAX): ");
    test_cannot_alloc(ptr_realloc_maxsize, realloc_maxsize_errno);
    free(ptr_realloc_maxsize);
    
    errno = 0;
    char * ptr_memalign_size0 = (char *)memalign(aligned_alloc_alignment, zero_size);
    int memalign_size0_errno = errno;
    printf("memalign (size 0): ");
    test_size_zero(ptr_memalign_size0, aligned_alloc_alignment, memalign_size0_errno);
    free(ptr_memalign_size0);
    
    errno = 0;
    char * ptr_memalign = (char *)memalign(aligned_alloc_alignment, sample_alloc_size);
    int memalign_errno = errno;
    printf("memalign: ");
    test_valid_aligned(ptr_memalign, aligned_alloc_alignment, memalign_errno);
    for(i = 0; i < sample_alloc_size; i++) {
        ptr_memalign[i] = (char)i;
    }
    free(ptr_memalign);
    
    errno = 0;
    char * ptr_memalign_maxsize = (char *)memalign(aligned_alloc_alignment, max_size);
    int memalign_maxsize_errno = errno;
    printf("memalign (SIZE_MAX): ");
    test_cannot_alloc(ptr_memalign_maxsize, memalign_maxsize_errno);
    free(ptr_memalign_maxsize);
    
    errno = 0;
    char * ptr_memalign_align0 = (char *)memalign(0, sample_alloc_size);
    int memalign_align0_errno = errno;
    printf("memalign (alignment 0): ");
    test_invalid_aligned(ptr_memalign_align0, memalign_align0_errno);
    free(ptr_memalign_align0);
    
    errno = 0;
    char * ptr_memalign_align3 = (char *)memalign(3, sample_alloc_size);
    int memalign_align3_errno = errno;
    printf("memalign (alignment 3): ");
    test_invalid_aligned(ptr_memalign_align3, memalign_align3_errno);
    free(ptr_memalign_align3);
    
    errno = 0;
    char * ptr_aligned_alloc_goodsize = (char *)aligned_alloc(aligned_alloc_alignment, aligned_alloc_goodsize);
    int aligned_alloc_goodsize_errno = errno;
    printf("aligned_alloc (size %% alignment == 0): ");
    test_valid_aligned(ptr_aligned_alloc_goodsize, aligned_alloc_alignment, aligned_alloc_goodsize_errno);
    free(ptr_aligned_alloc_goodsize);
    
    errno = 0;
    char * ptr_aligned_alloc_badsize = (char *)aligned_alloc(aligned_alloc_alignment, aligned_alloc_badsize);
    int aligned_alloc_badsize_errno = errno;
    printf("aligned_alloc (size %% alignment != 0): ");
    test_invalid_aligned(ptr_aligned_alloc_badsize, aligned_alloc_badsize_errno);
    free(ptr_aligned_alloc_badsize);
    
    errno = 0;
    char * ptr_valloc_size0 = (char *)valloc(zero_size);
    int valloc_size0_errno = errno;
    printf("valloc (size 0): ");
    test_size_zero(ptr_valloc_size0, page_size, valloc_size0_errno);
    free(ptr_valloc_size0);
    
    errno = 0;
    char * ptr_valloc = (char *)valloc(sample_alloc_size);
    int valloc_errno = errno;
    printf("valloc: ");
    test_valid_aligned(ptr_valloc, page_size, valloc_errno);
    free(ptr_valloc);
    
    errno = 0;
    char * ptr_valloc_maxsize = (char *)valloc(max_size);
    int valloc_maxsize_errno = errno;
    printf("valloc (SIZE_MAX): ");
    test_cannot_alloc(ptr_valloc_maxsize, valloc_maxsize_errno);
    free(ptr_valloc_maxsize);
    
    errno = 0;
    void * ptr_posix_memalign = NULL;
    int posix_memalign_return = posix_memalign(&ptr_posix_memalign, pow2_mul_voidptr_size, sample_alloc_size);
    printf("posix_memalign: ");
    test_valid_aligned(ptr_posix_memalign, pow2_mul_voidptr_size, posix_memalign_return);
    free(ptr_posix_memalign);
    
    errno = 0;
    void * ptr_posix_memalign_align0 = NULL;
    int posix_memalign_align0_return = posix_memalign(&ptr_posix_memalign_align0, zero_size, sample_alloc_size);
    printf("posix_memalign (alignment 0): ");
    test_invalid_aligned(ptr_posix_memalign_align0, posix_memalign_align0_return);
    free(ptr_posix_memalign_align0);
    
    errno = 0;
    void * ptr_posix_memalign_nonpow2mul = NULL;
    int posix_memalign_nonpow2mul_return = posix_memalign(&ptr_posix_memalign_nonpow2mul, nonpow2_mul_voidptr_size, sample_alloc_size);
    printf("posix_memalign (non-power-of-two multiple of sizeof(void *)): ");
    test_invalid_aligned(ptr_posix_memalign_nonpow2mul, posix_memalign_nonpow2mul_return);
    free(ptr_posix_memalign_nonpow2mul);
    
    errno = 0;
    void * ptr_posix_memalign_size0 = NULL;
    int posix_memalign_size0_return = posix_memalign(&ptr_posix_memalign_size0, pow2_mul_voidptr_size, zero_size);
    printf("posix_memalign (size 0): ");
    test_size_zero(ptr_posix_memalign_size0, pow2_mul_voidptr_size, posix_memalign_size0_return);
    free(ptr_posix_memalign_size0);
    
    errno = 0;
    void * ptr_posix_memalign_maxsize = NULL;
    int posix_memalign_maxsize_return = posix_memalign(&ptr_posix_memalign_maxsize, pow2_mul_voidptr_size, max_size);
    printf("posix_memalign (SIZE_MAX): ");
    test_cannot_alloc(ptr_posix_memalign_maxsize, posix_memalign_maxsize_return);
    free(ptr_posix_memalign_maxsize);
}
