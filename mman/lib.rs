pub extern "C" fn mlock(arg1: *const libc::c_void, arg2: usize)
     -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn mlockall(arg1: libc::c_int) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn mmap(arg1: *mut libc::c_void, arg2: usize,
                arg3: libc::c_int, arg4: libc::c_int,
                arg5: libc::c_int, arg6: off_t)
     -> *mut libc::c_void {
    unimplemented!();
}

pub extern "C" fn mprotect(arg1: *mut libc::c_void, arg2: usize,
                    arg3: libc::c_int) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn msync(arg1: *mut libc::c_void, arg2: usize,
                 arg3: libc::c_int) -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn munlock(arg1: *const libc::c_void, arg2: usize)
     -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn munlockall() -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn munmap(arg1: *mut libc::c_void, arg2: usize)
     -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn shm_open(arg1: *const libc::c_char,
                    arg2: libc::c_int, arg3: mode_t)
     -> libc::c_int {
    unimplemented!();
}

pub extern "C" fn shm_unlink(arg1: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

