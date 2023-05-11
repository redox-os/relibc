use super::*;

use crate::header::bits_pthread::pthread_attr_t;

impl Default for RlctAttr {
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
            },
            #[cfg(target_pointer_width = "32")]
            _pad: [0; 12],
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> c_int {
    core::ptr::drop_in_place(attr);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getdetachstate(attr: *const pthread_attr_t, detachstate: *mut c_int) -> c_int {
    core::ptr::write(detachstate, (*attr.cast::<RlctAttr>()).detachstate as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getguardsize(attr: *const pthread_attr_t, size: *mut size_t) -> c_int {
    core::ptr::write(size, (*attr.cast::<RlctAttr>()).guardsize);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getinheritsched(attr: *const pthread_attr_t, inheritsched: *mut c_int) -> c_int {
    core::ptr::write(inheritsched, (*attr.cast::<RlctAttr>()).inheritsched as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getschedparam(attr: *const pthread_attr_t, param: *mut sched_param) -> c_int {
    param.write((*attr.cast::<RlctAttr>()).param);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getschedpolicy(attr: *const pthread_attr_t, policy: *mut c_int) -> c_int {
    core::ptr::write(policy, (*attr.cast::<RlctAttr>()).schedpolicy as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getscope(attr: *const pthread_attr_t, scope: *mut c_int) -> c_int {
    core::ptr::write(scope, (*attr.cast::<RlctAttr>()).scope as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getstack(attr: *const pthread_attr_t, stackaddr: *mut *mut c_void, stacksize: *mut size_t) -> c_int {
    core::ptr::write(stackaddr, (*attr.cast::<RlctAttr>()).stack as _);
    core::ptr::write(stacksize, (*attr.cast::<RlctAttr>()).stacksize as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getstacksize(attr: *const pthread_attr_t, stacksize: *mut c_int) -> c_int {
    core::ptr::write(stacksize, (*attr.cast::<RlctAttr>()).stacksize as _);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int {
    core::ptr::write(attr.cast::<RlctAttr>(), RlctAttr::default());
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setdetachstate(attr: *mut pthread_attr_t, detachstate: c_int) -> c_int {
    (*attr.cast::<RlctAttr>()).detachstate = detachstate as _;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setguardsize(attr: *mut pthread_attr_t, guardsize: c_int) -> c_int {
    (*attr.cast::<RlctAttr>()).guardsize = guardsize as _;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inheritsched: c_int) -> c_int {
    (*attr.cast::<RlctAttr>()).inheritsched = inheritsched as _;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setschedparam(attr: *mut pthread_attr_t, param: *const sched_param) -> c_int {
    (*attr.cast::<RlctAttr>()).param = param.read();
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, policy: c_int) -> c_int {
    (*attr.cast::<RlctAttr>()).schedpolicy = policy as u8;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setscope(attr: *mut pthread_attr_t, scope: c_int) -> c_int {
    (*attr.cast::<RlctAttr>()).scope = scope as u8;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setstack(attr: *mut pthread_attr_t, stackaddr: *mut c_void, stacksize: size_t) -> c_int {
    (*attr.cast::<RlctAttr>()).stack = stackaddr as usize;
    (*attr.cast::<RlctAttr>()).stacksize = stacksize;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: size_t) -> c_int {
    (*attr.cast::<RlctAttr>()).stacksize = stacksize;
    0
}
