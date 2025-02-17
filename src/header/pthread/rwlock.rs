use super::*;

use crate::header::errno::EBUSY;

use crate::pthread::Pshared;

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_init(
    rwlock: *mut pthread_rwlock_t,
    attr: *const pthread_rwlockattr_t,
) -> c_int {
    let attr = unsafe { attr.cast::<RlctRwlockAttr>().as_ref() }
        .copied()
        .unwrap_or_default();

    let rwlock_value = RlctRwlock::new(attr.pshared);
    unsafe { rwlock.cast::<RlctRwlock>().write(rwlock_value) };

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    let rwlock = unsafe { &*rwlock.cast::<RlctRwlock>() };
    rwlock.acquire_read_lock(None);

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_timedrdlock(
    rwlock: *mut pthread_rwlock_t,
    timeout: *const timespec,
) -> c_int {
    let rwlock = unsafe { &*rwlock.cast::<RlctRwlock>() };
    let timeout = unsafe { &*timeout };
    rwlock.acquire_read_lock(Some(timeout));

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_timedwrlock(
    rwlock: *mut pthread_rwlock_t,
    timeout: *const timespec,
) -> c_int {
    let rwlock = unsafe { &*rwlock.cast::<RlctRwlock>() };
    let timeout = unsafe { &*timeout };
    rwlock.acquire_write_lock(Some(timeout));

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    let rwlock = unsafe { &*rwlock.cast::<RlctRwlock>() };
    match rwlock.try_acquire_read_lock() {
        Ok(()) => 0,
        Err(_) => EBUSY,
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    let rwlock = unsafe { &*rwlock.cast::<RlctRwlock>() };
    match rwlock.try_acquire_write_lock() {
        Ok(()) => 0,
        Err(_) => EBUSY,
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    let rwlock = unsafe { &*rwlock.cast::<RlctRwlock>() };
    rwlock.unlock();

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    let rwlock = unsafe { &*rwlock.cast::<RlctRwlock>() };
    rwlock.acquire_write_lock(None);

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> c_int {
    let attr_value = RlctRwlockAttr::default();
    unsafe { attr.cast::<RlctRwlockAttr>().write(attr_value) };

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_getpshared(
    attr: *const pthread_rwlockattr_t,
    pshared_out: *mut c_int,
) -> c_int {
    let attr = unsafe { &*attr.cast::<RlctRwlockAttr>() };
    let attr_raw = attr.pshared.raw();
    unsafe { core::ptr::write(pshared_out, attr_raw) };

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_setpshared(
    attr: *mut pthread_rwlockattr_t,
    pshared: c_int,
) -> c_int {
    let attr = unsafe { &mut *attr.cast::<RlctRwlockAttr>() };
    attr.pshared =
        Pshared::from_raw(pshared).expect("invalid pshared in pthread_rwlockattr_setpshared");

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_destroy(attr: *mut pthread_rwlockattr_t) -> c_int {
    unsafe { core::ptr::drop_in_place(attr) };

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_destroy(rwlock: *mut pthread_rwlock_t) -> c_int {
    unsafe { core::ptr::drop_in_place(rwlock) };

    0
}

pub(crate) type RlctRwlock = crate::sync::rwlock::InnerRwLock;

#[derive(Clone, Copy, Default)]
pub(crate) struct RlctRwlockAttr {
    pshared: Pshared,
}
