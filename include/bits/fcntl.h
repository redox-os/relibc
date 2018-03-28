#ifndef _BITS_FCNTL_H
#define _BITS_FCNTL_H

int open(const char* filename, int flags, ...);
int fcntl(int fildes, int cmd, ...);

#endif
