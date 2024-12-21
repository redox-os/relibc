//! `sys/auxv.h` implementation.
//!
//! Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/getauxval.3.html>.

use crate::platform::types::*;

pub use crate::platform::auxv_defs::*;

/// See <https://www.man7.org/linux/man-pages/man3/getauxval.3.html>.
#[no_mangle]
pub extern "C" fn getauxval(_t: c_ulong) -> c_ulong {
    0
}
