#ifndef _BITS_SYS_SELECT_H
#define _BITS_SYS_SELECT_H

#define FD_SETSIZE 1024
#define NFDBITS (8 * sizeof(unsigned long))

typedef struct fd_set {
    unsigned long fds_bits[FD_SETSIZE / NFDBITS];
} fd_set;

#define _FD_INDEX(fd) ((fd) / NFDBITS)
#define _FD_BITMASK(fd) (1UL << ((fd) & NFDBITS))

#define FD_ZERO(set) for (int i = 0; i < sizeof((set)->fds_bits) / sizeof(unsigned long); i += 1) { \
                         (set)->fds_bits[i] = 0; \
                     }

#define FD_SET(fd, set) ((set)->fds_bits[_FD_INDEX(fd)] |= _FD_BITMASK(fd))
#define FD_CLR(fd, set) ((set)->fds_bits[_FD_INDEX(fd)] &= ~(_FD_BITMASK(fd)))

#define FD_ISSET(fd, set) (((set)->fds_bits[_FD_INDEX(fd)] & _FD_BITMASK(fd)) == _FD_BITMASK(fd))

#endif
