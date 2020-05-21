//! wchar implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/wchar.h.html

use crate::platform::types::*;

mod casecmp;
use casecmp::casemap;
pub const WEOF: wint_t = 0xFFFF_FFFFu32;

#[no_mangle]
pub extern "C" fn towlower(wc: wint_t) -> wint_t {
    casemap(wc, 0)
}

#[no_mangle]
pub extern "C" fn towupper(wc: wint_t) -> wint_t {
    casemap(wc, 1)
}
