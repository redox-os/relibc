use crate::platform::types::c_int;

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
