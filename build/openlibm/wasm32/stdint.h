#pragma once

typedef unsigned char      uint8_t;
typedef unsigned short	   uint16_t;
typedef unsigned int       uint32_t;
typedef unsigned long long uint64_t;

typedef char               int8_t;
typedef short	           int16_t;
typedef int                int32_t;
typedef long long          int64_t;

typedef unsigned int       uintptr_t;
typedef int                intptr_t;

_Static_assert(sizeof (uint8_t)  == 1, "invalid size");
_Static_assert(sizeof (uint16_t) == 2, "invalid size");
_Static_assert(sizeof (uint32_t) == 4, "invalid size");
_Static_assert(sizeof (uint64_t) == 8, "invalid size");

_Static_assert(sizeof (int8_t)  == 1, "invalid size");
_Static_assert(sizeof (int16_t) == 2, "invalid size");
_Static_assert(sizeof (int32_t) == 4, "invalid size");
_Static_assert(sizeof (int64_t) == 8, "invalid size");

_Static_assert(sizeof (uintptr_t) == sizeof (intptr_t), "invalid size");
_Static_assert(sizeof (uintptr_t) == sizeof (void*), "invalid size");
_Static_assert(sizeof (uintptr_t) == 4, "invalid size");

#define UINT8_MAX  0xFF
#define UINT16_MAX 0xFFFF
#define UINT32_MAX 0xFFFFFFFFUL
#define UINT64_MAX 0xFFFFFFFFFFFFFFFFULL

#define INT8_MAX   0x7F
#define INT16_MAX  0x7FFF
#define INT32_MAX  0x7FFFFFFF
#define INT64_MAX  0x7FFFFFFFFFFFFFFF

#define INT8_MIN   (-0x80)
#define INT16_MIN  (-0x8000)
#define INT32_MIN  (-0x80000000L)
#define INT64_MIN  (-0x8000000000000000LL)
