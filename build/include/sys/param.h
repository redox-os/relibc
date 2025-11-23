#ifndef _SYS_PARAM_H
#define _SYS_PARAM_H

#define MIN(a,b) (((a) < (b)) ? (a) : (b))
#define MAX(a,b) (((a) > (b)) ? (a) : (b))

#define __bitop(array, index, op) ((array)[(index) / 8] op (1 << (index) % 8))
#define setbit(array, index) __bitop(array, index, |=)
#define clrbit(array, index) __bitop(array, index, &= ~)
#define isset(array, index) __bitop(array, index, &)
#define isclr(array, index) !isset(array, index)

#define howmany(bits, size) (((bits) + (size) - 1) / (size))
#define roundup(bits, size) (howmany(bits, size) * (size))
#define powerof2(n) !(((n) - 1) & (n))

// Shamelessly copied from musl.
// Tweak as needed.
#define MAXSYMLINKS 20
#define MAXHOSTNAMELEN 64
#define MAXNAMLEN 255
#define MAXPATHLEN 4096
#define NBBY 8
#define NGROUPS 32
#define CANBSIZ 255
#define NOFILE 256
#define NCARGS 131072
#define DEV_BSIZE 512
#define NOGROUP (-1)

#include <sys/resource.h>
#include <limits.h>

#include <machine/endian.h>

#endif
