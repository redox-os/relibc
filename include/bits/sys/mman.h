#ifndef _BITS_SYS_MMAN_H
#define _BITS_SYS_MMAN_H

#define MAP_FAILED ((void *) -1)

#if defined (_LARGEFILE64_SOURCE)
#define mmap64 mmap
#endif

#endif
