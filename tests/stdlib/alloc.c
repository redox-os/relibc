#include <malloc.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <stddef.h> /* for size_t */
#include <stdint.h> /* for SIZE_MAX */

#include "test_helpers.h"

int main(void) {
    size_t sample_alloc_size = 256;
    size_t sample_realloc_size = sample_alloc_size + 1;
    
    /* ensure values are mapped to variables */
    size_t zero = 0;
    size_t max_size = SIZE_MAX;
    size_t aligned_alloc_alignment = 128;
    size_t aligned_alloc_goodsize = 256;
    size_t aligned_alloc_badsize = 257;
    size_t nonpow2_mul_voidptr_size = 3*sizeof(void *);
    size_t pow2_mul_voidptr_size = 4*sizeof(void *);
    
    int i;
    
    errno = 0;
    char * ptr_malloc = (char *)malloc(sample_alloc_size);
    int malloc_errno = errno;
    printf("malloc                : %p, errno: %d = %s\n",
        ptr_malloc, malloc_errno, strerror(malloc_errno));
    for(i = 0; i < sample_alloc_size; i++) {
        ptr_malloc[i] = (char)i;
    }
    free(ptr_malloc);
    
    errno = 0;
    char * ptr_malloc_maxsize = (char *)malloc(max_size);
    int malloc_maxsize_errno = errno;
    printf("malloc (SIZE_MAX)     : %p, errno: %d = %s\n",
        ptr_malloc_maxsize, malloc_maxsize_errno,
        strerror(malloc_maxsize_errno));
    free(ptr_malloc_maxsize);
    
    errno = 0;
    char * ptr_calloc = (char *)calloc(sample_alloc_size, 1);
    int calloc_errno = errno;
    printf("calloc                : %p, errno: %d = %s\n", ptr_calloc,
        calloc_errno, strerror(calloc_errno));
    for(i = 0; i < sample_alloc_size; i++) {
        ptr_calloc[i] = (char)i;
    }
    free(ptr_calloc);
    
    errno = 0;
    char * ptr_calloc_overflow = (char *)calloc(max_size, max_size);
    int calloc_overflow_errno = errno;
    printf("calloc (overflowing)  : %p, errno: %d = %s\n",
        ptr_calloc_overflow, calloc_overflow_errno,
        strerror(calloc_overflow_errno));
    free(ptr_calloc_overflow); /* clean up correctly even if overflow is not handled */
    
    char * ptr_realloc = (char *)malloc(sample_alloc_size);
    errno = 0;
    ptr_realloc = (char *)realloc(ptr_realloc, sample_realloc_size);
    int realloc_errno = errno;
    printf("realloc               : %p, errno: %d = %s\n",
        ptr_realloc, realloc_errno, strerror(realloc_errno));
    for(i = 0; i < sample_realloc_size; i++) {
        ptr_realloc[i] = (char)i;
    }
    free(ptr_realloc);
    
    char * ptr_realloc_maxsize = (char *)malloc(sample_alloc_size);
    errno = 0;
    ptr_realloc_maxsize = (char *)realloc(ptr_realloc_maxsize, max_size);
    int realloc_maxsize_errno = errno;
    printf("realloc (SIZE_MAX)    : %p, errno: %d = %s\n",
        ptr_realloc_maxsize, realloc_maxsize_errno,
        strerror(realloc_maxsize_errno));
    free(ptr_realloc_maxsize);
    
    errno = 0;
    char * ptr_memalign = (char *)memalign(256, sample_alloc_size);
    int memalign_errno = errno;
    printf("memalign              : %p, errno: %d = %s\n", ptr_memalign,
        memalign_errno, strerror(memalign_errno));
    for(i = 0; i < sample_alloc_size; i++) {
        ptr_memalign[i] = (char)i;
    }
    free(ptr_memalign);
    
    errno = 0;
    char * ptr_memalign_maxsize = (char *)memalign(256, max_size);
    int memalign_maxsize_errno = errno;
    printf("memalign (SIZE_MAX)   : %p, errno: %d = %s\n",
        ptr_memalign_maxsize, memalign_maxsize_errno,
        strerror(memalign_maxsize_errno));
    free(ptr_memalign_maxsize);
    
    errno = 0;
    char * ptr_memalign_align0 = (char *)memalign(0, sample_alloc_size);
    int memalign_align0_errno = errno;
    printf("memalign (alignment 0): %p, errno: %d = %s\n",
        ptr_memalign_align0, memalign_align0_errno,
        strerror(memalign_align0_errno));
    free(ptr_memalign_align0);
    
    errno = 0;
    char * ptr_memalign_align3 = (char *)memalign(3, sample_alloc_size);
    int memalign_align3_errno = errno;
    printf("memalign (alignment 3): %p, errno: %d = %s\n",
        ptr_memalign_align3, memalign_align3_errno,
        strerror(memalign_align3_errno));
    free(ptr_memalign_align3);
    
    errno = 0;
    char * ptr_aligned_alloc_goodsize = (char *)aligned_alloc(aligned_alloc_alignment, aligned_alloc_goodsize);
    int aligned_alloc_goodsize_errno = errno;
    printf("aligned_alloc (size %% alignment == 0):\n");
    printf("                        %p, errno: %d = %s\n",
        ptr_aligned_alloc_goodsize, aligned_alloc_goodsize_errno,
        strerror(aligned_alloc_goodsize_errno));
    free(ptr_aligned_alloc_goodsize);
    
    errno = 0;
    char * ptr_aligned_alloc_badsize = (char *)aligned_alloc(aligned_alloc_alignment, aligned_alloc_badsize);
    int aligned_alloc_badsize_errno = errno;
    printf("aligned_alloc (size %% alignment != 0):\n");
    printf("                        %p, errno: %d = %s\n",
        ptr_aligned_alloc_badsize, aligned_alloc_badsize_errno,
        strerror(aligned_alloc_badsize_errno));
    free(ptr_aligned_alloc_badsize);
    
    errno = 0;
    char * ptr_valloc = (char *)valloc(sample_alloc_size);
    int valloc_errno = errno;
    printf("valloc                : %p, errno: %d = %s\n",
        ptr_valloc, valloc_errno, strerror(valloc_errno));
    
    errno = 0;
    char * ptr_valloc_maxsize = (char *)valloc(max_size);
    int valloc_maxsize_errno = errno;
    printf("valloc (SIZE_MAX)     : %p, errno: %d = %s\n",
        ptr_valloc_maxsize, valloc_maxsize_errno,
        strerror(valloc_maxsize_errno));
    
    errno = 0;
    void * ptr_posix_memalign = NULL;
    int posix_memalign_return = posix_memalign(&ptr_posix_memalign, pow2_mul_voidptr_size, sample_alloc_size);
    int posix_memalign_errno = errno;
    printf("posix_memalign:\n");
    printf("                        %p, return value: %d = %s,\n",
        ptr_posix_memalign,
        posix_memalign_return, strerror(posix_memalign_return));
    /* strerror() can only be called once in a printf call for correct
     * results */
    printf("                        errno: %d = %s\n",
        posix_memalign_errno,
        strerror(posix_memalign_errno));
    
    errno = 0;
    void * ptr_posix_memalign_align0 = NULL;
    int posix_memalign_align0_return = posix_memalign(&ptr_posix_memalign_align0, zero, sample_alloc_size);
    int posix_memalign_align0_errno = errno;
    printf("posix_memalign (alignment 0):\n");
    printf("                        %p, return value: %d = %s,\n",
        ptr_posix_memalign_align0,
        posix_memalign_align0_return,
        strerror(posix_memalign_align0_return));
    
    printf("                        errno: %d = %s\n",
        posix_memalign_align0_errno,
        strerror(posix_memalign_align0_errno));
    
    errno = 0;
    void * ptr_posix_memalign_nonpow2mul = NULL;
    int posix_memalign_nonpow2mul_return = posix_memalign(&ptr_posix_memalign_nonpow2mul, nonpow2_mul_voidptr_size, sample_alloc_size);
    int posix_memalign_nonpow2mul_errno = errno;
    printf("posix_memalign (non-power-of-two multiple of sizeof(void *)):\n");
    printf("                        %p, return value: %d = %s,\n",
        ptr_posix_memalign_nonpow2mul,
        posix_memalign_nonpow2mul_return,
        strerror(posix_memalign_nonpow2mul_return));
    
    printf("                        errno: %d = %s\n",
        posix_memalign_nonpow2mul_errno,
        strerror(posix_memalign_nonpow2mul_errno));
    
    errno = 0;
    void * ptr_posix_memalign_size0 = NULL;
    int posix_memalign_size0_return = posix_memalign(&ptr_posix_memalign_size0, pow2_mul_voidptr_size, zero);
    int posix_memalign_size0_errno = errno;
    printf("posix_memalign (size 0):\n");
    printf("                        %p, return value: %d = %s,\n",
        ptr_posix_memalign_size0,
        posix_memalign_size0_return,
        strerror(posix_memalign_size0_return));
    
    printf("                        errno: %d = %s\n",
        posix_memalign_size0_errno,
        strerror(posix_memalign_size0_errno));
    
    errno = 0;
    void * ptr_posix_memalign_maxsize = NULL;
    int posix_memalign_maxsize_return = posix_memalign(&ptr_posix_memalign_maxsize, pow2_mul_voidptr_size, max_size);
    int posix_memalign_maxsize_errno = errno;
    printf("posix_memalign (SIZE_MAX):\n");
    printf("                        %p, return value: %d = %s,\n",
        ptr_posix_memalign_maxsize,
        posix_memalign_maxsize_return,
        strerror(posix_memalign_maxsize_return));
    
    printf("                        errno: %d = %s\n",
        posix_memalign_maxsize_errno,
        strerror(posix_memalign_maxsize_errno));
}
