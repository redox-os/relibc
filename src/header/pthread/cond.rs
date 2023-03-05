use super::*;

// PTHREAD_COND_INITIALIZER

#[repr(C)]
pub struct CondAttr {
    clock: clockid_t,
    pshared: c_int,
}

#[repr(C)]
pub struct Cond {
}

// #[no_mangle]
pub extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    todo!()
}

// #[no_mangle]
pub extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    todo!()
}

// #[no_mangle]
pub extern "C" fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *const pthread_condattr_t) -> c_int {
    todo!()
}

// #[no_mangle]
pub extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    todo!()
}

// #[no_mangle]
pub extern "C" fn pthread_cond_timedwait(cond: *mut pthread_cond_t, mutex: *const pthread_mutex_t, timeout: *const timespec) -> c_int {
    todo!()
}

// #[no_mangle]
pub extern "C" fn pthread_cond_wait(cond: *mut pthread_cond_t, mutex: *const pthread_mutex_t) -> c_int {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_destroy(condattr: *mut pthread_condattr_t) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getclock(condattr: *const pthread_condattr_t, clock: *mut clockid_t) -> c_int {
    core::ptr::write(clock, (*condattr).clock);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getpshared(condattr: *const pthread_condattr_t, pshared: *mut c_int) -> c_int {
    core::ptr::write(pshared, (*condattr).pshared);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_init(condattr: *mut pthread_condattr_t) -> c_int {
    core::ptr::write(condattr, CondAttr {
        // FIXME: system clock
        clock: 0,
        // Default
        pshared: PTHREAD_PROCESS_PRIVATE,
    });
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setclock(condattr: *mut pthread_condattr_t, clock: clockid_t) -> c_int {
    (*condattr).clock = clock;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setpshared(condattr: *mut pthread_condattr_t, pshared: c_int) -> c_int {
    (*condattr).pshared = pshared;
    0
}

