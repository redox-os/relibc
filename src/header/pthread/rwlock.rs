use super::*;

use crate::header::errno::{EBUSY, EINVAL};

use crate::{header::time::CLOCK_REALTIME, pthread::Pshared};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_init.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_init(
    rwlock: *mut pthread_rwlock_t,
    attr: *const pthread_rwlockattr_t,
) -> c_int {
    let attr = unsafe { attr.cast::<RlctRwlockAttr>().as_ref() }
        .copied()
        .unwrap_or_default();

    unsafe {
        rwlock
            .cast::<RlctRwlock>()
            .write(RlctRwlock::new(attr.pshared))
    };

    0
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_rdlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    match unsafe { get(rwlock) }.acquire_read_lock(None) {
        Ok(()) => 0,
        Err(e) => e.0,
    }
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_clockrdlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_clockrdlock(
    rwlock: *mut pthread_rwlock_t,
    clock_id: clockid_t,
    abstime: *const timespec,
) -> c_int {
    match unsafe { get(rwlock) }.acquire_read_lock(Some((unsafe { &*abstime }, clock_id))) {
        Ok(()) => 0,
        Err(e) => e.0,
    }
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_timedrdlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_timedrdlock(
    rwlock: *mut pthread_rwlock_t,
    abstime: *const timespec,
) -> c_int {
    match unsafe { get(rwlock) }.acquire_read_lock(Some((unsafe { &*abstime }, CLOCK_REALTIME))) {
        Ok(()) => 0,
        Err(e) => e.0,
    }
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_clockwrlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_clockwrlock(
    rwlock: *mut pthread_rwlock_t,
    clock_id: clockid_t,
    abstime: *const timespec,
) -> c_int {
    match unsafe { get(rwlock) }.acquire_write_lock(Some((unsafe { &*abstime }, clock_id))) {
        Ok(()) => 0,
        Err(e) => e.0,
    }
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_timedwrlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_timedwrlock(
    rwlock: *mut pthread_rwlock_t,
    abstime: *const timespec,
) -> c_int {
    match unsafe { get(rwlock) }.acquire_write_lock(Some((unsafe { &*abstime }, CLOCK_REALTIME))) {
        Ok(()) => 0,
        Err(e) => e.0,
    }
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_tryrdlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    match unsafe { get(rwlock) }.try_acquire_read_lock() {
        Ok(()) => 0,
        Err(_) => EBUSY,
    }
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_trywrlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    match unsafe { get(rwlock) }.try_acquire_write_lock() {
        Ok(()) => 0,
        Err(_) => EBUSY,
    }
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_unlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    unsafe { get(rwlock) }.unlock();

    0
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_wrlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    match unsafe { get(rwlock) }.acquire_write_lock(None) {
        Ok(()) => 0,
        Err(e) => e.0,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlockattr_init.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> c_int {
    unsafe {
        attr.cast::<RlctRwlockAttr>()
            .write(RlctRwlockAttr::default())
    };

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlockattr_getpshared.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_getpshared(
    attr: *const pthread_rwlockattr_t,
    pshared_out: *mut c_int,
) -> c_int {
    unsafe { core::ptr::write(pshared_out, (*attr.cast::<RlctRwlockAttr>()).pshared.raw()) };

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlockattr_setpshared.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_setpshared(
    attr: *mut pthread_rwlockattr_t,
    pshared: c_int,
) -> c_int {
    let Some(pshared) = Pshared::from_raw(pshared) else {
        return EINVAL;
    };

    (unsafe { *attr.cast::<RlctRwlockAttr>() }).pshared = pshared;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlockattr_destroy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_destroy(attr: *mut pthread_rwlockattr_t) -> c_int {
    unsafe { core::ptr::drop_in_place(attr) };

    0
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_rwlock_destroy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_destroy(rwlock: *mut pthread_rwlock_t) -> c_int {
    unsafe { core::ptr::drop_in_place(rwlock) };

    0
}

pub(crate) type RlctRwlock = crate::sync::rwlock::InnerRwLock;

#[derive(Clone, Copy, Default)]
pub(crate) struct RlctRwlockAttr {
    pshared: Pshared,
}
#[inline]
unsafe fn get<'a>(ptr: *mut pthread_rwlock_t) -> &'a RlctRwlock {
    unsafe { &*ptr.cast() }
}
