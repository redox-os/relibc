#ifndef _BITS_FCNTL_H
#define _BITS_FCNTL_H

#if (defined(__redox__))
#define O_NOFOLLOW 0x80000000
#endif

#ifdef __cplusplus
extern "C" {
#endif

int open(const char* filename, int flags, ...);
int fcntl(int fildes, int cmd, ...);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
