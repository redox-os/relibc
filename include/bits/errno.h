#ifndef _BITS_ERRNO_H
#define _BITS_ERRNO_H

#ifdef __cplusplus
extern "C" {
#endif

#define ENOTSUP EOPNOTSUPP

#define errno (*__errno_location())

#ifdef __cplusplus
} // extern "C"
#endif

#endif /* _BITS_ERRNO_H */
