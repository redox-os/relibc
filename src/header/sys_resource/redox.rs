#[no_mangle]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlp: *mut rlimit) -> c_int {
    // TODO
    RLIM_INFINITY
}
