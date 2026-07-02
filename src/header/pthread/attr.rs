use core::{ptr, sync::atomic::Ordering};

use crate::{
    header::{
        bits_pthread::{pthread_attr_t, pthread_t},
        limits::PTHREAD_STACK_MIN,
        pthread::{
            PTHREAD_CREATE_DETACHED, PTHREAD_CREATE_JOINABLE, PTHREAD_EXPLICIT_SCHED,
            PTHREAD_INHERIT_SCHED, PTHREAD_SCOPE_PROCESS, PTHREAD_SCOPE_SYSTEM, RlctAttr,
        },
        sched::{SCHED_FIFO, SCHED_OTHER, SCHED_RR, sched_param},
    },
    platform::{
        Pal, Sys,
        types::{c_int, c_long, c_void, size_t},
    },
    pthread::{Pthread, PthreadFlags},
};

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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_destroy.html>.
///
/// Destroys a thread attributes object.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour for the following:
/// - `attr` is not already initialized when calling this function
/// - `attr` is used after calling this function without reinitializing
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> c_int {
    unsafe { ptr::drop_in_place(attr) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_getdetachstate.html>.
///
/// Gets the destachstate attribute in the `attr` object.
///
/// Upon success, returns `0` and stores the detachstate attribute in
/// `detachstate`. Upon failure, returns an error number to indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_getdetachstate(
    attr: *const pthread_attr_t,
    detachstate: *mut c_int,
) -> c_int {
    unsafe { ptr::write(detachstate, (*attr.cast::<RlctAttr>()).detachstate.into()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_getguardsize.html>.
///
/// Gets the guardsize attribute in the `attr` object.
///
/// Upon success, returns `0` and stores the guardsize attribute in the
/// `guardsize` parameter. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_getguardsize(
    attr: *const pthread_attr_t,
    guardsize: *mut size_t,
) -> c_int {
    unsafe { ptr::write(guardsize, (*attr.cast::<RlctAttr>()).guardsize) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_getinheritsched.html>.
///
/// Gets the inheritsched attribute in the `attr` object.
///
/// Upon success, returns `0` and stores the inheritsched attribute in the
/// `inheritsched` parameter. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_getinheritsched(
    attr: *const pthread_attr_t,
    inheritsched: *mut c_int,
) -> c_int {
    unsafe { ptr::write(inheritsched, (*attr.cast::<RlctAttr>()).inheritsched.into()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_getschedparam.html>.
///
/// Gets the scheduling parameter attributes in the `attr` object.
///
/// Upon success, returns `0` and stores the scheduling parameter attributes
/// in the `param` parameter. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_getschedparam(
    attr: *const pthread_attr_t,
    param: *mut sched_param,
) -> c_int {
    unsafe { param.write((*attr.cast::<RlctAttr>()).param) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_getschedpolicy.html>.
///
/// Gets the schedpolicy attribute in the `attr` object.
///
/// Upon success, returns `0` and stores the schedpolicy attributes in the
/// `policy` parameter. Upon failure, an error number is returned to indicate
/// the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_getschedpolicy(
    attr: *const pthread_attr_t,
    policy: *mut c_int,
) -> c_int {
    unsafe { ptr::write(policy, (*attr.cast::<RlctAttr>()).schedpolicy.into()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_getscope.html>.
///
/// Gets the contentionscope attribute in the `attr` object.
///
/// Upon success, returns `0` and stores the contentionscope attribute in the
/// `contentionscope` parameter. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_getscope(
    attr: *const pthread_attr_t,
    contentionscope: *mut c_int,
) -> c_int {
    unsafe { ptr::write(contentionscope, (*attr.cast::<RlctAttr>()).scope.into()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_getstack.html>.
///
/// Gets the thread creation stack attributes stackaddr and stacksize in the
/// `attr` object.
///
/// Upon success, returns `0` and stores the stackaddr attribute in the
/// `stackaddr` parameter and the stacksize attribute in the `stacksize`
/// parameter. Upon failure, an error number is returned to indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_getstack(
    attr: *const pthread_attr_t,
    stackaddr: *mut *mut c_void,
    stacksize: *mut size_t,
) -> c_int {
    unsafe { ptr::write(stackaddr, (*attr.cast::<RlctAttr>()).stack as _) };
    unsafe { ptr::write(stacksize, (*attr.cast::<RlctAttr>()).stacksize) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_getstacksize.html>.
///
/// Gets the thread creation stacksize attribute in the `attr` object.
///
/// Upon success, returns `0` and stores the stacksize attribute in the
/// `stacksize` parameter. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_getstacksize(
    attr: *const pthread_attr_t,
    stacksize: *mut size_t,
) -> c_int {
    unsafe { ptr::write(stacksize, (*attr.cast::<RlctAttr>()).stacksize) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_init.html>.
///
/// Initializes a thread attributes object `attr` with the default value for
/// all of the individual attributes used by a given implementation.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int {
    unsafe { ptr::write(attr.cast::<RlctAttr>(), RlctAttr::default()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_setdetachstate.html>.
///
/// Sets the detachstate attribute in the `attr` object.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_setdetachstate(
    attr: *mut pthread_attr_t,
    detachstate: c_int,
) -> c_int {
    match detachstate {
        PTHREAD_CREATE_DETACHED | PTHREAD_CREATE_JOINABLE => {
            // infallible, value of constants fit into `c_uchar`
            if let Ok(ds) = detachstate.try_into() {
                // SAFTEY: guaranteed to fit
                unsafe {
                    (*attr.cast::<RlctAttr>()).detachstate = ds;
                }
            }
            0
        }
        _ => crate::header::errno::EINVAL,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_setguardsize.html>.
///
/// Sets the guardsize attribute in the `attr` object.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_setguardsize(
    attr: *mut pthread_attr_t,
    guardsize: c_int,
) -> c_int {
    unsafe {
        (*attr.cast::<RlctAttr>()).guardsize = guardsize as _;
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_setinheritsched.html>.
///
/// Sets the inheritsched attribute in the `attr` object.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_setinheritsched(
    attr: *mut pthread_attr_t,
    inheritsched: c_int,
) -> c_int {
    match inheritsched {
        PTHREAD_INHERIT_SCHED | PTHREAD_EXPLICIT_SCHED => {
            // infallible, value of constants fit into `c_uchar`
            if let Ok(insch) = inheritsched.try_into() {
                // SAFTEY: guaranteed to fit
                unsafe {
                    (*attr.cast::<RlctAttr>()).inheritsched = insch;
                }
            }
            0
        }
        _ => crate::header::errno::ENOTSUP,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_setschedparam.html>.
///
/// Sets the scheduling parameter attributes in the `attr` object.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_setschedparam(
    attr: *mut pthread_attr_t,
    param: *const sched_param,
) -> c_int {
    unsafe {
        (*attr.cast::<RlctAttr>()).param = param.read();
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_setschedpolicy.html>.
///
/// Sets the schedpolicy attribute in the `attr` object.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_setschedpolicy(
    attr: *mut pthread_attr_t,
    policy: c_int,
) -> c_int {
    match policy {
        SCHED_FIFO | SCHED_OTHER | SCHED_RR => {
            // infallible, value of constants fit into `c_uchar`
            if let Ok(pol) = policy.try_into() {
                // SAFTEY: guaranteed to fit
                unsafe {
                    (*attr.cast::<RlctAttr>()).schedpolicy = pol;
                }
            }
            0
        }
        _ => crate::header::errno::ENOTSUP,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_setscope.html>.
///
/// Sets the contentionscope attribute in the `attr` object.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_setscope(
    attr: *mut pthread_attr_t,
    contentionscope: c_int,
) -> c_int {
    match contentionscope {
        PTHREAD_SCOPE_SYSTEM | PTHREAD_SCOPE_PROCESS => {
            // infallible, value of constants fit into `c_uchar`
            if let Ok(ctnscope) = contentionscope.try_into() {
                // SAFTEY: guaranteed to fit
                unsafe {
                    (*attr.cast::<RlctAttr>()).scope = ctnscope;
                }
            }
            0
        }
        _ => crate::header::errno::ENOTSUP,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_setstack.html>.
///
/// Sets the thread creation stack attributes stackaddr and stacksize in the
/// `attr` object.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_setstack(
    attr: *mut pthread_attr_t,
    stackaddr: *mut c_void,
    stacksize: size_t,
) -> c_int {
    if stacksize as c_long >= PTHREAD_STACK_MIN {
        unsafe {
            (*attr.cast::<RlctAttr>()).stack = stackaddr as usize;
        }
        unsafe {
            (*attr.cast::<RlctAttr>()).stacksize = stacksize;
        }
        0
    } else {
        crate::header::errno::EINVAL
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_attr_setstacksize.html>.
///
/// Sets the thread creation stacksize attribute in the `attr` object.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Safety
/// It is undefined behaviour if `attr` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_attr_setstacksize(
    attr: *mut pthread_attr_t,
    stacksize: size_t,
) -> c_int {
    if stacksize as c_long >= PTHREAD_STACK_MIN {
        unsafe {
            (*attr.cast::<RlctAttr>()).stacksize = stacksize;
        }
        0
    } else {
        crate::header::errno::EINVAL
    }
}

// TODO should be guarded by _GNU_SOURCE
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/pthread_getattr_np.3.html>.
///
/// Initializes the thread attributes object referenced to by `attr` so that it
/// contains actual attribute values describing the running thread `thread`.
///
/// Upon success, returns `0`. Upon failure, returns an error number to
/// indicate the error.
///
/// # Implementation
/// Always succeeds, so will never return an error number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_getattr_np(thread: pthread_t, attr: *mut pthread_attr_t) -> c_int {
    let thread = unsafe { &*thread.cast::<Pthread>() };
    let attr_ptr = attr.cast::<RlctAttr>();
    unsafe { ptr::write(attr_ptr, RlctAttr::default()) };
    let attr = unsafe { &mut *attr_ptr };
    if thread.flags.load(Ordering::Acquire) & PthreadFlags::DETACHED.bits() != 0 {
        attr.detachstate = PTHREAD_CREATE_DETACHED as _;
    }
    attr.stack = thread.stack_base as usize;
    attr.stacksize = thread.stack_size;
    //TODO: more values?
    0
}
