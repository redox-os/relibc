#ifndef _SYS_TYPES_H
#define _SYS_TYPES_H

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

typedef int useconds_t;

typedef long suseconds_t;

typedef long clock_t;

typedef int clockid_t;

typedef void* timer_t;

typedef unsigned long int blkcnt_t;

#ifdef __linux__
#define _SC_PAGE_SIZE 30
#endif
#ifdef __redox__
#define _SC_PAGE_SIZE 8
#endif

#endif /* _SYS_TYPES_H */
