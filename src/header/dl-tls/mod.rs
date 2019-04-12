//! dl-tls implementation for Redox

use platform::types::*;

#[repr(C)]
pub struct dl_tls_index {
    pub ti_module: u64,
    pub ti_offset: u64,
}

#[no_mangle]
pub extern "C" fn __tls_get_addr(ti: *mut dl_tls_index) -> *mut c_void {
    //TODO: Figure out how to implement this
    unimplemented!();
}
