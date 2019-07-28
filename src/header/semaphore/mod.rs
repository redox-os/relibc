use crate::platform::types::*;

#[repr(C)]
#[derive(Copy)]
pub union sem_t {
    pub size: [c_char; 32usize],
    pub align: c_long,
    _bindgen_union_align: [u64; 4usize],
}
impl Clone for sem_t {
    fn clone(&self) -> Self {
        *self
    }
}
// #[no_mangle]
pub extern "C" fn sem_init(sem: *mut sem_t, pshared: c_int, value: c_uint) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sem_destroy(sem: *mut sem_t) -> c_int {
    unimplemented!();
}

/*
 *#[no_mangle]
 *pub extern "C" fn sem_open(name: *const c_char,
 *                    oflag: c_int, ...) -> *mut sem_t {
 *    unimplemented!();
 *}
 */

// #[no_mangle]
pub extern "C" fn sem_close(sem: *mut sem_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sem_unlink(name: *const c_char) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sem_wait(sem: *mut sem_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sem_trywait(sem: *mut sem_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sem_post(sem: *mut sem_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn sem_getvalue(sem: *mut sem_t, sval: *mut c_int) -> c_int {
    unimplemented!();
}
