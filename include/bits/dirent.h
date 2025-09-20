#ifndef _BITS_DIRENT_H
#define _BITS_DIRENT_H

// Shamelessly stolen from musl
#define DT_UNKNOWN 0
#define DT_FIFO 1
#define DT_CHR 2
#define DT_DIR 4
#define DT_BLK 6
#define DT_REG 8
#define DT_LNK 10
#define DT_SOCK 12
#define DT_WHT 14
#define IFTODT(x) ((x)>>12 & 017)
#define DTTOIF(x) ((x)<<12)

// Shamelessly stolen from musl again
#if defined (_LARGEFILE64_SOURCE)
#define dirent64 dirent
#define readdir64 readdir
#define scandir64 scandir
#define alphasort64 alphasort
#define getdents64 getdents
#endif

#endif /* _BITS_DIRENT_H */
