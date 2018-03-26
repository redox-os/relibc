#[cfg(target_os = "redox")]
use syscall::data::TimeSpec as redox_timespec;
// Use repr(u8) as LLVM expects `void*` to be the same as `i8*` to help enable
// more optimization opportunities around it recognizing things like
// malloc/free.
#[repr(u8)]
pub enum c_void {
    // Two dummy variants so the #[repr] attribute can be used.
    #[doc(hidden)]
    __variant1,
    #[doc(hidden)]
    __variant2,
}

pub type int8_t = i8;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;

pub type c_schar = i8;
pub type c_uchar = u8;
pub type c_short = i16;
pub type c_ushort = u16;
pub type c_int = i32;
pub type c_uint = u32;
pub type c_float = f32;
pub type c_double = f64;
pub type c_longlong = i64;
pub type c_ulonglong = u64;
pub type intmax_t = i64;
pub type uintmax_t = u64;

pub type size_t = usize;
pub type ptrdiff_t = isize;
pub type intptr_t = isize;
pub type uintptr_t = usize;
pub type ssize_t = isize;

pub type c_char = i8;
pub type c_long = i64;
pub type c_ulong = u64;

pub type wchar_t = i16;
pub type wint_t = i32;
pub type wctype_t = i64;

pub type off_t = i64;
pub type mode_t = u16;
pub type time_t = i64;
pub type pid_t = usize;
pub type id_t = usize;
pub type gid_t = usize;
pub type uid_t = usize;
pub type dev_t = usize;
pub type ino_t = usize;
pub type nlink_t = usize;
pub type blksize_t = isize;

pub type useconds_t = i32;
pub type suseconds_t = i64;

pub type clock_t = i64;
pub type clockid_t = i32;
pub type timer_t = c_void;

#[repr(C)]
#[derive(Default)]
pub struct timespec {
    pub tv_sec: time_t,
    pub tv_nsec: c_long,
}

#[cfg(target_os = "redox")]
impl<'a> From<&'a timespec> for redox_timespec {
    fn from(tp: &timespec) -> redox_timespec {
        redox_timespec {
            tv_sec: tp.tv_sec,
            tv_nsec: tp.tv_nsec as i32,
        }
    }
}

#[repr(C)]
pub struct stat {
    pub st_dev: dev_t,
    pub st_ino: ino_t,
    pub st_nlink: nlink_t,
    pub st_mode: mode_t,
    pub st_uid: uid_t,
    pub st_gid: gid_t,
    pub st_rdev: dev_t,
    pub st_size: off_t,
    pub st_blksize: blksize_t,
    pub st_atim: time_t,
    pub st_mtim: time_t,
    pub st_ctim: time_t,
}
