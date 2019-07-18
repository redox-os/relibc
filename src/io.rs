pub use core_io::*;

use crate::platform;

pub fn last_os_error() -> Error {
    let errno = unsafe { platform::errno };
    Error::from_raw_os_error(errno)
}
