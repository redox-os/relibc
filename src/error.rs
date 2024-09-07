use crate::platform::types::c_int;

/// Positive error codes (EINVAL, not -EINVAL).
#[derive(Debug, Eq, PartialEq)]
// TODO: Move to a more generic place.
pub struct Errno(pub c_int);

#[cfg(target_os = "redox")]
impl From<syscall::Error> for Errno {
    fn from(value: syscall::Error) -> Self {
        Errno(value.errno)
    }
}
#[cfg(target_os = "redox")]
impl From<Errno> for syscall::Error {
    fn from(value: Errno) -> Self {
        syscall::Error::new(value.0)
    }
}

pub trait ResultExt<T> {
    fn or_minus_one_errno(self) -> T;
}
impl<T: From<i8>> ResultExt<T> for Result<T, Errno> {
    fn or_minus_one_errno(self) -> T {
        match self {
            Self::Ok(v) => v,
            Self::Err(Errno(errno)) => unsafe {
                crate::platform::ERRNO.set(errno);
                T::from(-1)
            },
        }
    }
}
