#ifndef _BITS_SYS_RESOURCE_H
#define _BITS_SYS_RESOURCE_H

#define	RUSAGE_SELF 0
#define	RUSAGE_CHILDREN (-1)
#define RUSAGE_BOTH (-2)
#define	RUSAGE_THREAD 1

#if defined (_LARGEFILE64_SOURCE)
#define getrlimit64 getrlimit
#define setrlimit64 setrlimit
#endif

#endif /* _BITS_SYS_RESOURCE_H */
