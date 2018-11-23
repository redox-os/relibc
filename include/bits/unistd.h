#ifndef _BITS_UNISTD_H
#define _BITS_UNISTD_H

#define _POSIX_VERSION 200809L

#ifdef __cplusplus
extern "C" {
#endif

int execl(const char *path, const char* argv0, ...);
int execle(const char *path, const char* argv0, ...);
int execlp(const char *file, const char* argv0, ...);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
