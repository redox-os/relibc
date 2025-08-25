/*
 * MIT License
 * Copyright (c) 2020 Rich Felker musl-libc
 */

#ifndef _FEATURES_H__RELIBC
#define _FEATURES_H__RELIBC

// Version metadata for feature gating
// This is useful for divergent implementation specific behavior
// glibc, ulibc, and likely others define a similar macro
// musl does not define an equivalent macro
#define __RELIBC__ 1
#define __RELIBC__MAJOR 0
#define __RELIBC__MINOR 2

/*
 * Sources:
 * https://en.cppreference.com/w/c/language/attributes
 * https://clang.llvm.org/docs/LanguageExtensions.html
 * https://gcc.gnu.org/onlinedocs/cpp/_005f_005fhas_005fc_005fattribute.html
 * https://gcc.gnu.org/onlinedocs/cpp/Standard-Predefined-Macros.html
 */

// Clang doesn't define __has_cpp_attribute if compiling C code
#if !defined(__has_cpp_attribute)
    #define __has_cpp_attribute(x) 0
#endif

// Clang doesn't define __has_c_attribute if compiling C++ code
#if !defined(__has_c_attribute)
    #define __has_c_attribute(x) 0
#endif

// Check if C23+ attributes are available
#if defined(__cplusplus)
// HACK: GCC backports C++ attributes to C++98 but doesn't accept attributes
// placed before the function like cbindgen emits.
// Let's just disable attributes for C++98 by checking if a random C++11
// feature is available.
#define __HAS_ATTRIBUTE(x) __cpp_variable_templates &&__has_cpp_attribute(x)
#else
#define __HAS_ATTRIBUTE(x)                                                     \
  (__has_c_attribute(x) || __STDC_VERSION__ >= 202311L ||                      \
   __has_cpp_attribute(x))
#endif

// TODO: Not emitted with cbindgen
#if __STDC_VERSION__ >= 199901L
    #define __restrict restrict
#elif !defined(__GNUC__)
    #define __restrict
#endif

// TODO: Not emitted with cbindgen
#if __STDC_VERSION__ >= 199901L || defined(__cplusplus)
    #define __inline inline
#elif !defined(__GNUC__)
    #define __inline
#endif

// Analogous to Rust's Never type
//TODO: clang fails to compile C with [[noreturn]]
#if defined(__cplusplus) && __HAS_ATTRIBUTE(noreturn)
    #define __noreturn [[noreturn]]
// #elif __STDC_VERSION__ >= 201112L
// FIXME: cbindgen incorrectly places _Noreturn
// #define __noreturn _Noreturn
#elif defined(__GNUC__)
    #define __noreturn __attribute__((__noreturn__))
#else
    #define __noreturn
#endif

// Analogous to Rust's #[must_use]
// C23 only
#if __HAS_ATTRIBUTE(nodiscard)
    #define __nodiscard [[nodiscard]]
    #define __nodiscardNote(x) [[nodiscard(x)]]
#else
    #define __nodiscard
    #define __nodiscardNote(x)
#endif

// Analogous to Rust's #[deprecated]
// C23 only
#if __HAS_ATTRIBUTE(deprecated)
    #define __deprecated [[deprecated]]
    #define __deprecatedNote(x) [[deprecated(x)]]
#else
    #define __deprecated
    #define __deprecatedNote(x)
#endif

#endif
