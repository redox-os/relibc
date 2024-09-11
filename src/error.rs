use alloc::boxed::Box;

use crate::{header::errno::STR_ERROR, platform::types::c_int};

/// Positive error codes (EINVAL, not -EINVAL).
#[derive(Debug, Eq, PartialEq)]
// TODO: Move to a more generic place.
pub struct Errno(pub c_int);

#[cfg(target_os = "redox")]
impl From<syscall::Error> for Errno {
    #[inline]
    fn from(value: syscall::Error) -> Self {
        Errno(value.errno)
    }
}
#[cfg(target_os = "redox")]
impl From<Errno> for syscall::Error {
    #[inline]
    fn from(value: Errno) -> Self {
        syscall::Error::new(value.0)
    }
}

impl From<Errno> for crate::io::Error {
    #[inline]
    fn from(Errno(errno): Errno) -> Self {
        Self::from_raw_os_error(errno)
    }
}

// TODO: core::error::Error

impl core::fmt::Display for Errno {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match usize::try_from(self.0).ok().and_then(|i| STR_ERROR.get(i)) {
            Some(desc) => write!(f, "{desc}"),
            None => write!(f, "unknown error ({})", self.0),
        }
    }
}

pub trait ResultExt<T> {
    fn or_minus_one_errno(self) -> T;
}
impl<T: From<i8>> ResultExt<T> for Result<T, Errno> {
    fn or_minus_one_errno(self) -> T {
        match self {
            Self::Ok(v) => v,
            Self::Err(Errno(errno)) => {
                crate::platform::ERRNO.set(errno);
                T::from(-1)
            }
        }
    }
}
pub trait ResultExtPtrMut<T> {
    fn or_errno_null_mut(self) -> *mut T;
}
impl<T> ResultExtPtrMut<T> for Result<*mut T, Errno> {
    fn or_errno_null_mut(self) -> *mut T {
        match self {
            Self::Ok(ptr) => ptr,
            Self::Err(Errno(errno)) => {
                crate::platform::ERRNO.set(errno);
                core::ptr::null_mut()
            }
        }
    }
}
impl<T> ResultExtPtrMut<T> for Result<Box<T>, Errno> {
    fn or_errno_null_mut(self) -> *mut T {
        match self {
            Self::Ok(ptr) => Box::into_raw(ptr),
            Self::Err(Errno(errno)) => {
                crate::platform::ERRNO.set(errno);
                core::ptr::null_mut()
            }
        }
    }
}
