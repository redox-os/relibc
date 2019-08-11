#[no_mangle]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlp: *mut rlimit) -> c_int {
    Sys::getrlimit(resource, rlp)
}
