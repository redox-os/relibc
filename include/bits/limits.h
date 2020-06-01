#ifndef _BITS_LIMIT_H
#define _BITS_LIMIT_H

#define CHAR_BIT __CHAR_BIT__
#ifdef __CHAR_MAX__
# define CHAR_MAX __CHAR_MAX__
#else
# define CHAR_MAX 0xFF
#endif
#define CHAR_MIN 0
#define INT_MAX __INT_MAX__
#define INT_MIN (-INT_MAX - 1)
#define LLONG_MAX __LONG_LONG_MAX__
#define LLONG_MIN (-LLONG_MAX - 1)
#define LONG_BIT __LONG_WIDTH__
#define LONG_MAX __LONG_MAX__
#define LONG_MIN (-LONG_MAX - 1)
#define SCHAR_MAX __SCHAR_MAX__
#define SCHAR_MIN (-SCHAR_MAX - 1)
#define SHRT_MAX __SHRT_MAX__
#define SHRT_MIN (-SHRT_MAX - 1)

// TODO: These might not be accurate on all platforms
#define SSIZE_MAX 0x7fffffffffffffff
#define UCHAR_MAX 255
#define UINT_MAX 0xffffffff
#define ULLONG_MAX 0xffffffffffffffff
#define ULONG_MAX 0xffffffffffffffff
#define USHRT_MAX 0xffffffffffffffff
#define WORD_BIT 32

#endif
