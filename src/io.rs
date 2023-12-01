pub use core_io::*;

use crate::{errno::Errno, platform};

impl From<Error> for Errno {
    fn from(err: Error) -> Self {
        Errno(err.raw_os_error().unwrap())
    }
}

pub fn last_os_error() -> Error {
    let errno = unsafe { platform::errno };
    Error::from_raw_os_error(errno)
}
