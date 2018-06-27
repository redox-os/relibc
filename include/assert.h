#ifndef _ASSERT_H
#define _ASSERT_H

#ifdef NDEBUG
# define assert(cond)
#else
# include <stdio.h>
# define assert(cond) if (!(cond)) { \
    fprintf(stderr, "%s: %s:%d: Assertion `%s` failed.\n", __func__, __FILE__, __LINE__, #cond); \
    abort(); \
    }
#endif

#endif
