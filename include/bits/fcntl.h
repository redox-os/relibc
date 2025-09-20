#ifndef _BITS_FCNTL_H
#define _BITS_FCNTL_H

#if defined (_LARGEFILE64_SOURCE)
#define F_GETLK64 F_GETLK
#define F_SETLK64 F_SETLK
#define F_SETLKW64 F_SETLKW

#define flock64 flock
#define open64 open
#endif

#endif
