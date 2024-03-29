#ifndef _SYS_REDOX_H
#define _SYS_REDOX_H

#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifdef __redox__

ssize_t redox_fpath(int fd, void * buf, size_t count);
void * redox_physalloc(size_t size);
int redox_physfree(void * physical_address, size_t size);

#endif

#ifdef __cplusplus
} // extern "C"
#endif

#endif
