#ifndef _SYS_USER_H
#define _SYS_USER_H
#if defined(__amd64__) || defined(__amd64) || defined(__x86_64__) || defined(__x86_64) || defined(_M_AMD64)
#include <arch/x64/user.h>
#elif defined(__aarch64__)
#include <arch/aarch64/user.h>
#else
#error "Unknown architecture"
#endif

#endif