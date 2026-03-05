use super::*;

use crate::header::errno::{EBUSY, EINVAL};

use crate::pthread::Pshared;

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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    unsafe { get(rwlock) }.acquire_read_lock(None);

    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_timedrdlock(
    rwlock: *mut pthread_rwlock_t,
    timeout: *const timespec,
) -> c_int {
    unsafe { get(rwlock) }.acquire_read_lock(Some(unsafe { &*timeout }));

    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_timedwrlock(
    rwlock: *mut pthread_rwlock_t,
    timeout: *const timespec,
) -> c_int {
    unsafe { get(rwlock) }.acquire_write_lock(Some(unsafe { &*timeout }));

    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    match unsafe { get(rwlock) }.try_acquire_read_lock() {
        Ok(()) => 0,
        Err(_) => EBUSY,
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    match unsafe { get(rwlock) }.try_acquire_write_lock() {
        Ok(()) => 0,
        Err(_) => EBUSY,
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    unsafe { get(rwlock) }.unlock();

    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    unsafe { get(rwlock) }.acquire_write_lock(None);

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> c_int {
    unsafe {
        attr.cast::<RlctRwlockAttr>()
            .write(RlctRwlockAttr::default())
    };

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_getpshared(
    attr: *const pthread_rwlockattr_t,
    pshared_out: *mut c_int,
) -> c_int {
    unsafe { core::ptr::write(pshared_out, (*attr.cast::<RlctRwlockAttr>()).pshared.raw()) };

    0
}

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_destroy(attr: *mut pthread_rwlockattr_t) -> c_int {
    unsafe { core::ptr::drop_in_place(attr) };

    0
}
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
