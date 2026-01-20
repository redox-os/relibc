//! `semaphore.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/semaphore.h.html>.

use crate::{
    header::time::{CLOCK_MONOTONIC, CLOCK_REALTIME, timespec},
    platform::types::{c_char, c_int, c_long, c_uint, clockid_t},
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/semaphore.h.html>.
// TODO: Statically verify size and align
#[repr(C)]
#[derive(Clone, Copy)]
pub union sem_t {
    pub size: [c_char; 4],
    pub align: c_long,
}
pub type RlctSempahore = crate::sync::Semaphore;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_close.html>.
// #[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_close(sem: *mut sem_t) -> c_int {
    todo!("named semaphores")
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_destroy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_destroy(sem: *mut sem_t) -> c_int {
    unsafe { core::ptr::drop_in_place(sem.cast::<RlctSempahore>()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_getvalue.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_getvalue(sem: *mut sem_t, sval: *mut c_int) -> c_int {
    unsafe { sval.write(get(sem).value() as c_int) };

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_init.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_init(sem: *mut sem_t, _pshared: c_int, value: c_uint) -> c_int {
    unsafe { sem.cast::<RlctSempahore>().write(RlctSempahore::new(value)) };

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_open.html>.
// TODO: va_list
// #[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_open(
    name: *const c_char,
    oflag: c_int, /* (va_list) value: c_uint */
) -> *mut sem_t {
    todo!("named semaphores")
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_post.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_post(sem: *mut sem_t) -> c_int {
    unsafe { get(sem) }.post(1);

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_trywait.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_trywait(sem: *mut sem_t) -> c_int {
    unsafe { get(sem) }.try_wait();

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_unlink.html>.
// #[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_unlink(name: *const c_char) -> c_int {
    todo!("named semaphores")
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_trywait.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_wait(sem: *mut sem_t) -> c_int {
    unsafe { get(sem) }.wait(None, CLOCK_MONOTONIC);

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_clockwait.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_clockwait(
    sem: *mut sem_t,
    clock_id: clockid_t,
    abstime: *const timespec,
) -> c_int {
    unsafe { get(sem) }.wait(Some(&unsafe { *abstime }), clock_id);

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sem_timedwait.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sem_timedwait(sem: *mut sem_t, abstime: *const timespec) -> c_int {
    unsafe { get(sem) }.wait(Some(&unsafe { *abstime }), CLOCK_REALTIME);

    0
}

unsafe fn get<'any>(sem: *mut sem_t) -> &'any RlctSempahore {
    unsafe { &*sem.cast() }
}
