#[repr(C)]
#[derive(Copy)]
pub union sem_t {
    pub size: [libc::c_char; 32usize],
    pub align: libc::c_long,
    _bindgen_union_align: [u64; 4usize],
}
impl Clone for sem_t {
    fn clone(&self) -> Self { *self }
}
#[no_mangle]
pub extern "C" fn sem_init(sem: *mut sem_t, pshared: libc::c_int,
                    value: libc::c_uint) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sem_destroy(sem: *mut sem_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sem_open(name: *const libc::c_char,
                    oflag: libc::c_int, ...) -> *mut sem_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sem_close(sem: *mut sem_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sem_unlink(name: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sem_wait(sem: *mut sem_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sem_timedwait(sem: *mut sem_t, abstime: *const timespec)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sem_trywait(sem: *mut sem_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sem_post(sem: *mut sem_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sem_getvalue(sem: *mut sem_t, sval: *mut libc::c_int)
     -> libc::c_int {
    unimplemented!();
}

