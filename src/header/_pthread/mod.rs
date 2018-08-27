use platform::types::*;

#[repr(C)]
#[derive(Debug, Copy)]
pub struct sched_param {
    pub _address: u8,
}
impl Clone for sched_param {
    fn clone(&self) -> Self {
        *self
    }
}

// #[no_mangle]
pub extern "C" fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getdetachstate(
    attr: *const pthread_attr_t,
    detachstate: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getguardsize(
    attr: *const pthread_attr_t,
    guardsize: *mut usize,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getinheritsched(
    attr: *const pthread_attr_t,
    inheritsched: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getschedparam(
    attr: *const pthread_attr_t,
    param: *mut sched_param,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getschedpolicy(
    attr: *const pthread_attr_t,
    policy: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getscope(
    attr: *const pthread_attr_t,
    contentionscope: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getstackaddr(
    attr: *const pthread_attr_t,
    stackaddr: *mut *mut c_void,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getstacksize(
    attr: *const pthread_attr_t,
    stacksize: *mut usize,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_init(arg1: *mut pthread_attr_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setdetachstate(
    attr: *mut pthread_attr_t,
    detachstate: c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setguardsize(arg1: *mut pthread_attr_t, arg2: usize) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setinheritsched(
    attr: *mut pthread_attr_t,
    inheritsched: c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setschedparam(
    attr: *mut pthread_attr_t,
    param: *mut sched_param,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setschedpolicy(
    attr: *mut pthread_attr_t,
    policy: c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setscope(
    attr: *mut pthread_attr_t,
    contentionscope: c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setstackaddr(
    attr: *mut pthread_attr_t,
    stackaddr: *mut c_void,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: usize) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cancel(thread: pthread_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cleanup_push(routine: *mut c_void, arg: *mut c_void) {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cleanup_pop(execute: c_int) {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_init(
    cond: *mut pthread_cond_t,
    attr: *const pthread_condattr_t,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    abstime: *const timespec,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_wait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_condattr_destroy(attr: *mut pthread_condattr_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_condattr_getpshared(
    attr: *const pthread_condattr_t,
    pshared: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_condattr_init(attr: *mut pthread_condattr_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_condattr_setpshared(
    attr: *mut pthread_condattr_t,
    pshared: c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_create(
    thread: *mut pthread_t,
    attr: *const pthread_attr_t,
    start_routine: Option<unsafe extern "C" fn(arg1: *mut c_void) -> *mut c_void>,
    arg: *mut c_void,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_detach(thread: pthread_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_equal(t1: pthread_t, t2: pthread_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_exit(value_ptr: *mut c_void) {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_getconcurrency() -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_getschedparam(
    thread: pthread_t,
    policy: *mut c_int,
    param: *mut sched_param,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_getspecific(key: pthread_key_t) -> *mut c_void {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_join(thread: pthread_t, value_ptr: *mut *mut c_void) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_key_create(
    key: *mut pthread_key_t,
    destructor: Option<unsafe extern "C" fn(arg1: *mut c_void)>,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_key_delete(key: pthread_key_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_getprioceiling(
    mutex: *const pthread_mutex_t,
    prioceiling: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_init(
    mutex: *mut pthread_mutex_t,
    attr: *const pthread_mutexattr_t,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_setprioceiling(
    mutex: *mut pthread_mutex_t,
    prioceiling: c_int,
    old_ceiling: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_getprioceiling(
    attr: *const pthread_mutexattr_t,
    prioceiling: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_getprotocol(
    attr: *const pthread_mutexattr_t,
    protocol: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_getpshared(
    attr: *const pthread_mutexattr_t,
    pshared: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_gettype(
    attr: *const pthread_mutexattr_t,
    type_: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_setprioceiling(
    attr: *mut pthread_mutexattr_t,
    prioceiling: c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_setprotocol(
    attr: *mut pthread_mutexattr_t,
    protocol: c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_setpshared(
    attr: *mut pthread_mutexattr_t,
    pshared: c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_settype(
    attr: *mut pthread_mutexattr_t,
    type_: c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_once(
    once_control: *mut pthread_once_t,
    init_routine: Option<unsafe extern "C" fn()>,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_destroy(rwlock: *mut pthread_rwlock_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_init(
    rwlock: *mut pthread_rwlock_t,
    attr: *const pthread_rwlockattr_t,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_rdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_unlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_wrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlockattr_destroy(rwlock: *mut pthread_rwlockattr_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlockattr_getpshared(
    rwlock: *const pthread_rwlockattr_t,
    pshared: *mut c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlockattr_init(rwlock: *mut pthread_rwlockattr_t) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlockattr_setpshared(
    rwlock: *mut pthread_rwlockattr_t,
    pshared: c_int,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_self() -> pthread_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_setcancelstate(state: c_int, oldstate: *mut c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_setcanceltype(type_: c_int, oldtype: *mut c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_setconcurrency(new_level: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_setschedparam(
    thread: pthread_t,
    policy: c_int,
    param: *mut sched_param,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_setspecific(
    key: pthread_key_t,
    value: *const c_void,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_testcancel() {
    unimplemented!();
}
