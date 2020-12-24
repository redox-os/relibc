#ifndef _BITS_ERRNO_H
#define _BITS_ERRNO_H

#ifdef __cplusplus
extern "C" {
#endif

#define ENOTSUP EOPNOTSUPP

#define errno (*__errno_location())
#define program_invocation_name (*__program_invocation_name())
#define program_invocation_short_name (*__program_invocation_short_name())

#ifdef __cplusplus
} // extern "C"
#endif

#endif /* _BITS_ERRNO_H */
