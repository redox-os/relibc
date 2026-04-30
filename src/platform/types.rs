//! C data types for this platform.

// Use repr(u8) as LLVM expects `void*` to be the same as `i8*` to help enable
// more optimization opportunities around it recognizing things like
// malloc/free.
/// The `void` type in C.
#[repr(u8)]
pub enum c_void {
    // Two dummy variants so the #[repr] attribute can be used.
    #[doc(hidden)]
    __variant1,
    #[doc(hidden)]
    __variant2,
}

/// The `int8_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type int8_t = i8;
/// The `int16_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type int16_t = i16;
/// The `int32_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type int32_t = i32;
/// The `int64_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type int64_t = i64;
/// The `uint8_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type uint8_t = u8;
/// The `uint16_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type uint16_t = u16;
/// The `uint32_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type uint32_t = u32;
/// The `uint64_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type uint64_t = u64;

/// The `signed char` type in C.
pub type c_schar = i8;
/// The `unsigned char` type in C.
pub type c_uchar = u8;
/// The `short` type in C.
pub type c_short = i16;
/// The `unsigned short` type in C.
pub type c_ushort = u16;
/// The `int` type in C.
pub type c_int = i32;
/// The `unsigned int` type in C.
pub type c_uint = u32;
/// The `float` type in C.
pub type c_float = f32;
/// The `double` type in C.
pub type c_double = f64;
/// The `long long` type in C.
pub type c_longlong = i64;
/// The `unsigned long long` type in C.
pub type c_ulonglong = u64;
/// The `intmax_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type intmax_t = i64;
/// The `uintmax_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type uintmax_t = u64;

/// The `size_t` type provided in `stddef.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stddef.h.html>.
pub type size_t = usize;
/// The `ptrdiff_t` type provided in `stddef.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stddef.h.html>.
pub type ptrdiff_t = isize;
/// The `intptr_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type intptr_t = isize;
/// The `uintptr_t` type provided in `stdint.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdint.h.html>.
pub type uintptr_t = usize;
/// The `ssize_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type ssize_t = isize;

/// The `char` type in C.
pub type c_char = core::ffi::c_char;
/// The `long` type in C.
#[cfg(target_pointer_width = "32")]
pub type c_long = i32;
/// The `unsigned long` type in C.
#[cfg(target_pointer_width = "32")]
pub type c_ulong = u32;
/// The `long` type in C.
#[cfg(target_pointer_width = "64")]
pub type c_long = i64;
/// The `unsigned long` type in C.
#[cfg(target_pointer_width = "64")]
pub type c_ulong = u64;

/// The `wchar_t` type provided in `stddef.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stddef.h.html>.
pub type wchar_t = i32;
/// The `wint_t` type provided in [`wchar.h`](crate::header::wchar).
pub type wint_t = u32;

/// The `regoff_t` type provided in [`regex.h`](crate::header::regex).
pub type regoff_t = size_t;
/// The `off_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type off_t = c_longlong;
/// The `mode_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[cfg(target_os = "linux")]
pub type mode_t = c_uint;
/// The `mode_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[cfg(not(target_os = "linux"))]
pub type mode_t = c_int;
/// The `time_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type time_t = c_longlong;
/// The `pid_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type pid_t = c_int;
/// The `id_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type id_t = c_uint;
/// The `gid_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type gid_t = c_int;
/// The `uid_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type uid_t = c_int;
/// The `dev_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type dev_t = c_ulonglong;
/// The `ino_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type ino_t = c_ulonglong;
/// The `reclen_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type reclen_t = c_ushort;
/// The `nlink_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type nlink_t = c_ulong;
/// The `blksize_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type blksize_t = c_long;
/// The `blkcnt_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type blkcnt_t = c_longlong;

/// The `fsblkcnt_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type fsblkcnt_t = c_ulong;
/// The `fsfilcnt_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type fsfilcnt_t = c_ulong;

/// The `useconds_t` type provided in [`sys/types.h`](crate::header::sys_types) prior to Issue 7.
#[deprecated]
pub type useconds_t = c_uint;
/// The `suseconds_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[cfg(target_os = "linux")]
pub type suseconds_t = c_long;
// TODO: Should we break this to c_long as well? This also breaks timeval as well
//       but it will be consistent with timespec.tv_nsec (note that syscall already uses c_int)
/// The `suseconds_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[cfg(not(target_os = "linux"))]
pub type suseconds_t = c_int;

/// The `clock_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type clock_t = c_long;
/// The `clockid_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type clockid_t = c_int;
/// The `timer_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type timer_t = *mut c_void;

// A C long double is 96 bit in x86, 128 bit in other 64-bit targets
// However, both in x86 and x86_64 is actually f80 padded which rust has no underlying support,
//     while aarch64 (and possibly riscv64) support full f128 type but behind a feature gate.
// Until rust supporting them, relibc will lose precision to get them working, plus:
//     All read operation to this type must be converted from "relibc_ldtod".
//     All write operation to this type must be converted with "relibc_dtold".
/// The `long double` type in C.
#[cfg(target_pointer_width = "64")]
pub type c_longdouble = u128;
/// The `long double` type in C.
#[cfg(target_pointer_width = "32")]
pub type c_longdouble = [u32; 3];

pub use crate::header::bits_pthread::*;

/// The `max_align_t` type provided in `stddef.h`, see <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stddef.h.html>.
#[repr(C, align(16))]
pub struct max_align_t {
    _priv: [f64; 4],
}
