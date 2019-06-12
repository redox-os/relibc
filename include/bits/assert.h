#ifndef _BITS_ASSERT_H
#define _BITS_ASSERT_H

#ifdef NDEBUG
# define assert(cond) (void) 0
#else
# define assert(cond) \
  ((void)((cond) || (__assert_fail(__func__, __FILE__, __LINE__, #cond), 0)))
#endif

#endif
