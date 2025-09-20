#ifndef _BITS_UNISTD_H
#define _BITS_UNISTD_H

#define _POSIX_VERSION 200809L
#define _POSIX_REALTIME_SIGNALS 202405L
#define _XOPEN_VERSION 700L

#if defined (_LARGEFILE64_SOURCE)
#define ftruncate64 ftruncate
#define truncate64 truncate

#define lockf64 lockf
#define lseek64 lseek
#endif

#endif
