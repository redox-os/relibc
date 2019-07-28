//! sys/auxv.h implementation

use crate::platform::types::*;

pub const AT_HWCAP: usize = 16;

#[no_mangle]
pub extern "C" fn getauxval(_t: c_ulong) -> c_ulong {
    0
}
