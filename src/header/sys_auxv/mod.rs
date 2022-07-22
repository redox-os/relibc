//! sys/auxv.h implementation

use crate::platform::types::*;

pub use crate::platform::auxv_defs::*;

#[no_mangle]
pub extern "C" fn getauxval(_t: c_ulong) -> c_ulong {
    0
}
