#ifndef _BITS_STATVFS_H
#define _BITS_STATVFS_H

#if defined (_LARGEFILE64_SOURCE)
#define statvfs64 statvfs
#define fstatvfs64 fstatvfs
#endif

#endif
