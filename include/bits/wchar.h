#ifndef _BITS_WCHAR_H
#define _BITS_WCHAR_H

// int32_t, uint32_t, WCHAR_MIN, WCHAR_MAX
#include <stdint.h>

#ifndef _WCHAR_T
#define _WCHAR_T
    #ifndef __WCHAR_TYPE__
        #define __WCHAR_TYPE__ int32_t
    #endif
    typedef __WCHAR_TYPE__ wchar_t;
#endif // _WCHAR_T

#ifndef _WINT_T
#define _WINT_T
    #ifndef __WINT_TYPE__
        #define __WINT_TYPE__ uint32_t
    #endif
    typedef __WINT_TYPE__ wint_t;
#endif // _WINT_T

// NULL, size_t, must come after wchar_t and wint_t
#define __need_size_t
#define __need_NULL
#include <stddef.h>

#define WEOF (0xffffffffu)

#endif /* _BITS_WCHAR_H */
