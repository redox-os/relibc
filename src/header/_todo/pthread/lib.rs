// #[no_mangle]
pub extern "C" fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getdetachstate(
    attr: *const pthread_attr_t,
    detachstate: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getguardsize(
    attr: *const pthread_attr_t,
    guardsize: *mut usize,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getinheritsched(
    attr: *const pthread_attr_t,
    inheritsched: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getschedparam(
    attr: *const pthread_attr_t,
    param: *mut sched_param,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getschedpolicy(
    attr: *const pthread_attr_t,
    policy: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getscope(
    attr: *const pthread_attr_t,
    contentionscope: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getstackaddr(
    attr: *const pthread_attr_t,
    stackaddr: *mut *mut libc::c_void,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_getstacksize(
    attr: *const pthread_attr_t,
    stacksize: *mut usize,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_init(arg1: *mut pthread_attr_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setdetachstate(
    attr: *mut pthread_attr_t,
    detachstate: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setguardsize(arg1: *mut pthread_attr_t, arg2: usize) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setinheritsched(
    attr: *mut pthread_attr_t,
    inheritsched: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setschedparam(
    attr: *mut pthread_attr_t,
    param: *mut sched_param,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setschedpolicy(
    attr: *mut pthread_attr_t,
    policy: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setscope(
    attr: *mut pthread_attr_t,
    contentionscope: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setstackaddr(
    attr: *mut pthread_attr_t,
    stackaddr: *mut libc::c_void,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, stacksize: usize) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cancel(thread: pthread_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cleanup_push(routine: *mut libc::c_void, arg: *mut libc::c_void) {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cleanup_pop(execute: libc::c_int) {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_init(
    cond: *mut pthread_cond_t,
    attr: *const pthread_condattr_t,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    abstime: *const timespec,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_cond_wait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_condattr_destroy(attr: *mut pthread_condattr_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_condattr_getpshared(
    attr: *const pthread_condattr_t,
    pshared: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_condattr_init(attr: *mut pthread_condattr_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_condattr_setpshared(
    attr: *mut pthread_condattr_t,
    pshared: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_create(
    thread: *mut pthread_t,
    attr: *const pthread_attr_t,
    start_routine: Option<unsafe extern "C" fn(arg1: *mut libc::c_void) -> *mut libc::c_void>,
    arg: *mut libc::c_void,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_detach(thread: pthread_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_equal(t1: pthread_t, t2: pthread_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_exit(value_ptr: *mut libc::c_void) {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_getconcurrency() -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_getschedparam(
    thread: pthread_t,
    policy: *mut libc::c_int,
    param: *mut sched_param,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_getspecific(key: pthread_key_t) -> *mut libc::c_void {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_join(thread: pthread_t, value_ptr: *mut *mut libc::c_void) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_key_create(
    key: *mut pthread_key_t,
    destructor: Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_key_delete(key: pthread_key_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_getprioceiling(
    mutex: *const pthread_mutex_t,
    prioceiling: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_init(
    mutex: *mut pthread_mutex_t,
    attr: *const pthread_mutexattr_t,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_setprioceiling(
    mutex: *mut pthread_mutex_t,
    prioceiling: libc::c_int,
    old_ceiling: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_getprioceiling(
    attr: *const pthread_mutexattr_t,
    prioceiling: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_getprotocol(
    attr: *const pthread_mutexattr_t,
    protocol: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_getpshared(
    attr: *const pthread_mutexattr_t,
    pshared: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_gettype(
    attr: *const pthread_mutexattr_t,
    type_: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_setprioceiling(
    attr: *mut pthread_mutexattr_t,
    prioceiling: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_setprotocol(
    attr: *mut pthread_mutexattr_t,
    protocol: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_setpshared(
    attr: *mut pthread_mutexattr_t,
    pshared: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutexattr_settype(
    attr: *mut pthread_mutexattr_t,
    type_: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_once(
    once_control: *mut pthread_once_t,
    init_routine: Option<unsafe extern "C" fn()>,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_destroy(rwlock: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_init(
    rwlock: *mut pthread_rwlock_t,
    attr: *const pthread_rwlockattr_t,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_rdlock(rwlock: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_unlock(rwlock: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlock_wrlock(rwlock: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlockattr_destroy(rwlock: *mut pthread_rwlockattr_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlockattr_getpshared(
    rwlock: *const pthread_rwlockattr_t,
    pshared: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlockattr_init(rwlock: *mut pthread_rwlockattr_t) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_rwlockattr_setpshared(
    rwlock: *mut pthread_rwlockattr_t,
    pshared: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_self() -> pthread_t {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_setcancelstate(state: libc::c_int, oldstate: *mut libc::c_int) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_setcanceltype(type_: libc::c_int, oldtype: *mut libc::c_int) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_setconcurrency(new_level: libc::c_int) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_setschedparam(
    thread: pthread_t,
    policy: libc::c_int,
    param: *mut sched_param,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_setspecific(
    key: pthread_key_t,
    value: *const libc::c_void,
) -> libc::c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn pthread_testcancel() {
    unimplemented!();
}

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
