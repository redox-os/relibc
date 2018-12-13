use core::slice;

use platform::types::*;

use super::e;

#[no_mangle]
pub unsafe extern "C" fn redox_fpath(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    e(syscall::fpath(fd as usize, slice::from_raw_parts_mut(buf as *mut u8, count))) as ssize_t
}
