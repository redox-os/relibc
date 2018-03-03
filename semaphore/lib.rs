#[repr(C)]
#[derive(Copy)]
pub union sem_t {
    pub __size: [libc::c_char; 32usize],
    pub __align: libc::c_long,
    _bindgen_union_align: [u64; 4usize],
}
impl Clone for sem_t {
    fn clone(&self) -> Self { *self }
}
pub extern "C" fn sem_init(__sem: *mut sem_t, __pshared: libc::c_int,
                    __value: libc::c_uint) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn sem_destroy(__sem: *mut sem_t) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn sem_open(__name: *const libc::c_char,
                    __oflag: libc::c_int, ...) -> *mut sem_t {
    unimplemented!();
}

pub extern "C" fn sem_close(__sem: *mut sem_t) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn sem_unlink(__name: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn sem_wait(__sem: *mut sem_t) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn sem_timedwait(__sem: *mut sem_t, __abstime: *const timespec)
     -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn sem_trywait(__sem: *mut sem_t) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn sem_post(__sem: *mut sem_t) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn sem_getvalue(__sem: *mut sem_t, __sval: *mut libc::c_int)
     -> libc::c_int {
    unimplemented!();
}

