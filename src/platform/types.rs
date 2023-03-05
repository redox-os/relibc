use core::i32;

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
#[cfg(target_pointer_width = "32")]
pub type c_long = i32;
#[cfg(target_pointer_width = "32")]
pub type c_ulong = u32;
#[cfg(target_pointer_width = "64")]
pub type c_long = i64;
#[cfg(target_pointer_width = "64")]
pub type c_ulong = u64;

pub type wchar_t = i32;
pub type wint_t = u32;

pub type regoff_t = size_t;
pub type off_t = c_longlong;
pub type mode_t = c_int;
pub type time_t = c_longlong;
pub type pid_t = c_int;
pub type id_t = c_uint;
pub type gid_t = c_int;
pub type uid_t = c_int;
pub type dev_t = c_long;
pub type ino_t = c_ulong;
pub type nlink_t = c_ulong;
pub type blksize_t = c_long;
pub type blkcnt_t = c_ulong;

pub type fsblkcnt_t = c_ulong;
pub type fsfilcnt_t = c_ulong;

pub type useconds_t = c_uint;
pub type suseconds_t = c_int;

pub type clock_t = c_long;
pub type clockid_t = c_int;
pub type timer_t = *mut c_void;

pub type pthread_attr_t = crate::header::pthread::attr::Attr;
pub type pthread_barrier_t = crate::header::pthread::barrier::Barrier;
pub type pthread_barrierattr_t = crate::header::pthread::barrier::BarrierAttr;
pub type pthread_cond_t = crate::header::pthread::cond::Cond;
pub type pthread_condattr_t = crate::header::pthread::cond::CondAttr;
pub type pthread_key_t = *mut c_void;
pub type pthread_mutex_t = crate::header::pthread::mutex::Mutex;
pub type pthread_mutexattr_t = crate::header::pthread::mutex::MutexAttr;
pub type pthread_once_t = crate::header::pthread::once::Once;
pub type pthread_rwlock_t = crate::header::pthread::rwlock::Rwlock;
pub type pthread_rwlockattr_t = crate::header::pthread::rwlock::RwlockAttr;
pub type pthread_spinlock_t = crate::header::pthread::spin::Spinlock;
pub type pthread_t = *mut c_void;
