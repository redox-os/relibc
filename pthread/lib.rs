#[no_mangle]
pub extern "C" fn pthread_attr_destroy(arg1: *mut pthread_attr_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_getdetachstate(
    arg1: *const pthread_attr_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_getguardsize(
    arg1: *const pthread_attr_t,
    arg2: *mut usize,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_getinheritsched(
    arg1: *const pthread_attr_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_getschedparam(
    arg1: *const pthread_attr_t,
    arg2: *mut sched_param,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_getschedpolicy(
    arg1: *const pthread_attr_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_getscope(
    arg1: *const pthread_attr_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_getstackaddr(
    arg1: *const pthread_attr_t,
    arg2: *mut *mut libc::c_void,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_getstacksize(
    arg1: *const pthread_attr_t,
    arg2: *mut usize,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_init(arg1: *mut pthread_attr_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_setdetachstate(
    arg1: *mut pthread_attr_t,
    arg2: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_setguardsize(arg1: *mut pthread_attr_t, arg2: usize) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_setinheritsched(
    arg1: *mut pthread_attr_t,
    arg2: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_setschedparam(
    arg1: *mut pthread_attr_t,
    arg2: *mut sched_param,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_setschedpolicy(
    arg1: *mut pthread_attr_t,
    arg2: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_setscope(
    arg1: *mut pthread_attr_t,
    arg2: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_setstackaddr(
    arg1: *mut pthread_attr_t,
    arg2: *mut libc::c_void,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_attr_setstacksize(arg1: *mut pthread_attr_t, arg2: usize) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_cancel(arg1: pthread_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_cleanup_push(arg1: *mut libc::c_void, arg2: *mut libc::c_void) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_cleanup_pop(arg1: libc::c_int) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_cond_broadcast(arg1: *mut pthread_cond_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_cond_destroy(arg1: *mut pthread_cond_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_cond_init(
    arg1: *mut pthread_cond_t,
    arg2: *const pthread_condattr_t,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_cond_signal(arg1: *mut pthread_cond_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_cond_timedwait(
    arg1: *mut pthread_cond_t,
    arg2: *mut pthread_mutex_t,
    arg3: *const timespec,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_cond_wait(
    arg1: *mut pthread_cond_t,
    arg2: *mut pthread_mutex_t,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_condattr_destroy(arg1: *mut pthread_condattr_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_condattr_getpshared(
    arg1: *const pthread_condattr_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_condattr_init(arg1: *mut pthread_condattr_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_condattr_setpshared(
    arg1: *mut pthread_condattr_t,
    arg2: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_create(
    arg1: *mut pthread_t,
    arg2: *const pthread_attr_t,
    arg3: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void) -> *mut libc::c_void>,
    arg4: *mut libc::c_void,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_detach(arg1: pthread_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_equal(arg1: pthread_t, arg2: pthread_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_exit(arg1: *mut libc::c_void) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_getconcurrency() -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_getschedparam(
    arg1: pthread_t,
    arg2: *mut libc::c_int,
    arg3: *mut sched_param,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_getspecific(arg1: pthread_key_t) -> *mut libc::c_void {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_join(arg1: pthread_t, arg2: *mut *mut libc::c_void) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_key_create(
    arg1: *mut pthread_key_t,
    arg2: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_key_delete(arg1: pthread_key_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutex_destroy(arg1: *mut pthread_mutex_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutex_getprioceiling(
    arg1: *const pthread_mutex_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutex_init(
    arg1: *mut pthread_mutex_t,
    arg2: *const pthread_mutexattr_t,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutex_lock(arg1: *mut pthread_mutex_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutex_setprioceiling(
    arg1: *mut pthread_mutex_t,
    arg2: libc::c_int,
    arg3: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutex_trylock(arg1: *mut pthread_mutex_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutex_unlock(arg1: *mut pthread_mutex_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_destroy(arg1: *mut pthread_mutexattr_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_getprioceiling(
    arg1: *const pthread_mutexattr_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_getprotocol(
    arg1: *const pthread_mutexattr_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_getpshared(
    arg1: *const pthread_mutexattr_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_gettype(
    arg1: *const pthread_mutexattr_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_init(arg1: *mut pthread_mutexattr_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_setprioceiling(
    arg1: *mut pthread_mutexattr_t,
    arg2: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_setprotocol(
    arg1: *mut pthread_mutexattr_t,
    arg2: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_setpshared(
    arg1: *mut pthread_mutexattr_t,
    arg2: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_settype(
    arg1: *mut pthread_mutexattr_t,
    arg2: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_once(
    arg1: *mut pthread_once_t,
    arg2: ::std::option::Option<unsafe extern "C" fn()>,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlock_destroy(arg1: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlock_init(
    arg1: *mut pthread_rwlock_t,
    arg2: *const pthread_rwlockattr_t,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlock_rdlock(arg1: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlock_tryrdlock(arg1: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlock_trywrlock(arg1: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlock_unlock(arg1: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlock_wrlock(arg1: *mut pthread_rwlock_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlockattr_destroy(arg1: *mut pthread_rwlockattr_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlockattr_getpshared(
    arg1: *const pthread_rwlockattr_t,
    arg2: *mut libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlockattr_init(arg1: *mut pthread_rwlockattr_t) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_rwlockattr_setpshared(
    arg1: *mut pthread_rwlockattr_t,
    arg2: libc::c_int,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_self() -> pthread_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_setcancelstate(arg1: libc::c_int, arg2: *mut libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_setcanceltype(arg1: libc::c_int, arg2: *mut libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_setconcurrency(arg1: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_setschedparam(
    arg1: pthread_t,
    arg2: libc::c_int,
    arg3: *mut sched_param,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pthread_setspecific(
    arg1: pthread_key_t,
    arg2: *const libc::c_void,
) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
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
