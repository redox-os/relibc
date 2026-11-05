pub use crate::header::bits_nl_item::nl_item;
use crate::{
    error::{Errno, Result, ResultExt},
    header::errno::EOPNOTSUPP,
    platform::{
        self,
        types::{c_char, c_int},
    },
};

/// Used by the message catalogue functions to
/// identify a catalogue descriptor.
pub type nl_catd = c_int;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn catopen(_name: *const c_char, _oflags: c_int) -> nl_catd {
    let res: Result<nl_catd> = Err(Errno(EOPNOTSUPP));
    res.or_minus_one_errno()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn catclose(_catd: nl_catd) -> c_int {
    platform::ERRNO.set(EOPNOTSUPP);
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn catgets(
    _catd: nl_catd,
    _set_id: c_int,
    _msg_id: c_int,
    s: *const c_char,
) -> *const c_char {
    platform::ERRNO.set(EOPNOTSUPP);
    s
}
