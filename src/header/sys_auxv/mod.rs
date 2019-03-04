//! sys/auxv.h implementation

use platform::types::*;
use platform::{Pal, Sys};

pub const AT_HWCAP: usize = 16;

#[no_mangle]
pub extern "C" fn getauxval(_t: c_ulong) -> c_ulong {
    0
}
