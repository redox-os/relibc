use crate::platform::types::*;

#[no_mangle]
pub unsafe extern "C" fn pvalloc(size: size_t) -> *mut c_void {
    unimplemented!();
}
