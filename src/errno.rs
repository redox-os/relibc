use core::ptr;

use crate::{
    io,
    platform::types::{c_char, c_int, c_void},
};

/// Positive error codes (EINVAL, not -EINVAL).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Errno(pub c_int);
// TODO
// pub type Errno = NonZeroU32;
//
impl Errno {
    pub fn get(self) -> c_int {
        let Errno(err) = self;
        err as c_int
    }

    pub fn set_errno(self) -> c_int {
        let Errno(err) = self;
        unsafe { crate::platform::errno = err as c_int };
        err
    }
}

impl From<Errno> for io::Error {
    fn from(errno: Errno) -> Self {
        io::Error::from_raw_os_error(errno.get())
    }
}

pub trait IntoPThread {
    fn into_pthread_style(self) -> c_int;
}

impl IntoPThread for Result<(), Errno> {
    fn into_pthread_style(self) -> c_int {
        match self {
            Ok(_) => 0,
            Err(err) => err.get(),
        }
    }
}

pub trait IntoPosix<T> {
    fn into_posix_style(self) -> T;
}

macro_rules! into_posix {
    ($type:ident) => {
        impl IntoPosix<$type> for Result<$type, Errno> {
            fn into_posix_style(self) -> $type {
                match self {
                    Ok(val) => val as $type,
                    Err(err) => {
                        err.set_errno();
                        -1 as $type
                    }
                }
            }
        }
    };
    ($type:ident, true) => {
        impl IntoPosix<*mut $type> for Result<*mut $type, Errno> {
            fn into_posix_style(self) -> *mut $type {
                match self {
                    Ok(val) => val as *mut $type,
                    Err(err) => {
                        err.set_errno();
                        ptr::null_mut() as *mut $type
                    }
                }
            }
        }
    };
}

into_posix!(c_int);
into_posix!(isize);
into_posix!(i64);
into_posix!(c_void, true);
into_posix!(c_char, true);

impl IntoPosix<c_int> for Result<(), Errno> {
    fn into_posix_style(self) -> c_int {
        match self {
            Ok(_) => 0,
            Err(err) => {
                err.set_errno();
                -1
            }
        }
    }
}
