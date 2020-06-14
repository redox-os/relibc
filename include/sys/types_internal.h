#ifndef _SYS_TYPES_INTERNAL_H
#define _SYS_TYPES_INTERNAL_H
#include <stddef.h>

typedef long blksize_t;
typedef long dev_t;
typedef unsigned long ino_t;
typedef int gid_t;
typedef int uid_t;
typedef int mode_t;
typedef unsigned long nlink_t;
typedef long off_t;
typedef int pid_t;
typedef unsigned id_t;
typedef long ssize_t;
typedef long time_t;
typedef unsigned int useconds_t;
typedef int suseconds_t;
typedef long clock_t;
typedef int clockid_t;
typedef void* timer_t;
typedef unsigned long int blkcnt_t;

typedef unsigned long int fsblkcnt_t;
typedef unsigned long int fsfilcnt_t;

typedef unsigned char u_char, uchar;
typedef unsigned short u_short, ushort;
typedef unsigned int u_int, uint;
typedef unsigned long u_long, ulong;
typedef long long quad_t;
typedef unsigned long long u_quad_t;
typedef char *caddr_t;
#endif /* _SYS_TYPES_INTERNAL_H */
