#ifndef _BITS_UNISTD_H
#define _BITS_UNISTD_H

#define _POSIX_VERSION 200809L

int execl(const char *path, const char* argv0, ...);
int execle(const char *path, const char* argv0, ...);

#endif
