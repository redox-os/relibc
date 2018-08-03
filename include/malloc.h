#ifndef _MALLOC_H
#define _MALLOC_H

#include <stddef.h>

// Generated from:
// `grep "malloc\|calloc\|realloc\|free\|valloc\|memalign" target/include/stdlib.h`

void *calloc(size_t nelem, size_t elsize);
void free(void *ptr);
void *malloc(size_t size);
void *memalign(size_t alignment, size_t size);
void *realloc(void *ptr, size_t size);
void *valloc(size_t size);

#endif
