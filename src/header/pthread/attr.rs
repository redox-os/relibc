use super::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Attr {
    pub detachstate: u8,
    pub inheritsched: u8,
    pub schedpolicy: u8,
    pub scope: u8,
    pub guardsize: usize,
    pub stacksize: usize,
    pub stack: usize,
    pub param: sched_param,
}
impl Default for Attr {
    fn default() -> Self {
        Self {
            // Default according to POSIX.
            detachstate: PTHREAD_CREATE_JOINABLE as _,
            // Default according to POSIX.
            inheritsched: PTHREAD_INHERIT_SCHED as _,
            // TODO: Linux
            // Redox uses a round-robin scheduler
            schedpolicy: SCHED_RR as _,
            // TODO: Linux uses this one.
            scope: PTHREAD_SCOPE_SYSTEM as _,
            guardsize: Sys::getpagesize(),
            // TODO
            stack: 0,
            // TODO
            stacksize: 1024 * 1024,
            param: sched_param {
                // TODO
                sched_priority: 0,
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_destroy(_attr: *mut pthread_attr_t) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getdetachstate(attr: *const pthread_attr_t, detachstate: *mut c_int) -> c_int {
    core::ptr::write(detachstate, (*attr).detachstate as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getguardsize(attr: *const pthread_attr_t, size: *mut size_t) -> c_int {
    core::ptr::write(size, (*attr).guardsize);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getinheritsched(attr: *const pthread_attr_t, inheritsched: *mut c_int) -> c_int {
    core::ptr::write(inheritsched, (*attr).inheritsched as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getschedparam(attr: *const pthread_attr_t, param: *mut sched_param) -> c_int {
    core::ptr::write(param, (*attr).param);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getschedpolicy(attr: *const pthread_attr_t, policy: *mut c_int) -> c_int {
    core::ptr::write(policy, (*attr).schedpolicy as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getscope(attr: *const pthread_attr_t, scope: *mut c_int) -> c_int {
    core::ptr::write(scope, (*attr).scope as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getstack(attr: *const pthread_attr_t, stackaddr: *mut *mut c_void, stacksize: *mut size_t) -> c_int {
    core::ptr::write(stackaddr, (*attr).stack as _);
    core::ptr::write(stacksize, (*attr).stacksize as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getstacksize(attr: *const pthread_attr_t, stacksize: *mut c_int) -> c_int {
    core::ptr::write(stacksize, (*attr).stacksize as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int {
    core::ptr::write(attr, Attr::default());
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setdetachstate(attr: *mut pthread_attr_t, detachstate: c_int) -> c_int {
    (*attr).detachstate = detachstate as _;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setguardsize(attr: *mut pthread_attr_t, guardsize: c_int) -> c_int {
    (*attr).guardsize = guardsize as _;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: c_int) -> c_int {
    (*attr).inheritsched = inheritsched as _;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setschedparam(attr: *mut pthread_attr_t, param: *const sched_param) -> c_int {
    (*attr).param = core::ptr::read(param);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: c_int) -> c_int {
    (*attr).schedpolicy = policy as u8;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setscope(attr: *mut pthread_attr_t, scope: c_int) -> c_int {
    (*attr).scope = scope as u8;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setstack(attr: *mut pthread_attr_t, stackaddr: *mut c_void, stacksize: size_t) -> c_int {
    (*attr).stack = stackaddr as usize;
    (*attr).stacksize = stacksize;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: size_t) -> c_int {
    (*attr).stacksize = stacksize;
    0
}
