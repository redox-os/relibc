#ifndef _BITS_ASSERT_H
#define _BITS_ASSERT_H

#ifdef NDEBUG
# define assert(cond)
#else
# define assert(cond) if (!(cond)) { \
    __assert(__func__, __FILE__, __LINE__, #cond); \
  }
#endif

#endif
