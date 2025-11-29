use super::*;

use crate::header::errno::EBUSY;

use crate::pthread::Pshared;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_init(
    rwlock: *mut pthread_rwlock_t,
    attr: *const pthread_rwlockattr_t,
) -> c_int {
    guard_null!(rwlock);
    let attr = attr
        .cast::<RlctRwlockAttr>()
        .as_ref()
        .copied()
        .unwrap_or_default();

    rwlock
        .cast::<RlctRwlock>()
        .write(RlctRwlock::new(attr.pshared));

    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    guard_null!(rwlock);
    get(rwlock).acquire_read_lock(None);

    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_timedrdlock(
    rwlock: *mut pthread_rwlock_t,
    timeout: *const timespec,
) -> c_int {
    guard_null!(rwlock);
    get(rwlock).acquire_read_lock(Some(&*timeout));

    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_timedwrlock(
    rwlock: *mut pthread_rwlock_t,
    timeout: *const timespec,
) -> c_int {
    guard_null!(rwlock);
    get(rwlock).acquire_write_lock(Some(&*timeout));

    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    guard_null!(rwlock);
    match get(rwlock).try_acquire_read_lock() {
        Ok(()) => 0,
        Err(_) => EBUSY,
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    guard_null!(rwlock);
    match get(rwlock).try_acquire_write_lock() {
        Ok(()) => 0,
        Err(_) => EBUSY,
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    guard_null!(rwlock);
    get(rwlock).unlock();

    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    guard_null!(rwlock);
    get(rwlock).acquire_write_lock(None);

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> c_int {
    guard_null!(attr);
    attr.cast::<RlctRwlockAttr>()
        .write(RlctRwlockAttr::default());

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_getpshared(
    attr: *const pthread_rwlockattr_t,
    pshared_out: *mut c_int,
) -> c_int {
    guard_null!(attr);
    core::ptr::write(pshared_out, (*attr.cast::<RlctRwlockAttr>()).pshared.raw());

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_setpshared(
    attr: *mut pthread_rwlockattr_t,
    pshared: c_int,
) -> c_int {
    guard_null!(attr);
    (*attr.cast::<RlctRwlockAttr>()).pshared =
        Pshared::from_raw(pshared).expect("invalid pshared in pthread_rwlockattr_setpshared");

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlockattr_destroy(attr: *mut pthread_rwlockattr_t) -> c_int {
    guard_null!(attr);
    core::ptr::drop_in_place(attr);

    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_rwlock_destroy(rwlock: *mut pthread_rwlock_t) -> c_int {
    guard_null!(rwlock);
    core::ptr::drop_in_place(rwlock);

    0
}

pub(crate) type RlctRwlock = crate::sync::rwlock::InnerRwLock;

#[derive(Clone, Copy, Default)]
pub(crate) struct RlctRwlockAttr {
    pshared: Pshared,
}
#[inline]
unsafe fn get<'a>(ptr: *mut pthread_rwlock_t) -> &'a RlctRwlock {
    &*ptr.cast()
}
