use super::*;

use crate::sync::AtomicLock;
use core::sync::atomic::Ordering;

use crate::header::errno::EBUSY;

// PTHREAD_RWLOCK_INITIALIZER

// TODO: Optimize for short waits and long waits, using AtomicLock::wait_until, but still
// supporting timeouts.
// TODO: Add futex ops that use bitmasks.

const EXCLUSIVE: u32 = (1 << (u32::BITS - 1)) - 1;
// Separate "waiting for wrlocks" and "waiting for rdlocks"?
//const WAITING: u32 = 1 << (u32::BITS - 1);

#[repr(C)]
pub struct Rwlock {
    state: AtomicLock,
}

#[repr(C)]
pub struct RwlockAttr {
    pshared: c_int,
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_destroy(rwlock: *mut pthread_rwlock_t) -> c_int {
    // (Informing the compiler that this pointer is valid, might improve optimizations.)
    let _rwlock: &pthread_rwlock_t = &*rwlock;
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_init(rwlock: *mut pthread_rwlock_t, _attr: *const pthread_rwlockattr_t) -> c_int {
    core::ptr::write(rwlock, Rwlock {
        state: AtomicLock::new(0),
    });

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_rdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    pthread_rwlock_timedrdlock(rwlock, core::ptr::null())
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_timedrdlock(rwlock: *mut pthread_rwlock_t, timeout: *const timespec) -> c_int {
    let rwlock: &pthread_rwlock_t = &*rwlock;
    let timeout = timeout.as_ref();

    loop {
        if pthread_rwlock_tryrdlock(rwlock as *const _ as *mut _) == EBUSY {
            rwlock.state.wait_if(EXCLUSIVE as i32, timeout);
        }
        return 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_timedwrlock(rwlock: *mut pthread_rwlock_t, timeout: *const timespec) -> c_int {
    let rwlock: &pthread_rwlock_t = &*rwlock;
    let timeout = timeout.as_ref();

    loop {
        if pthread_rwlock_trywrlock(rwlock as *const _ as *mut _) == EBUSY {
            rwlock.state.wait_if(EXCLUSIVE as i32, timeout);
        }
        return 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_tryrdlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    let rwlock: &pthread_rwlock_t = &*rwlock;

    let mut cached = rwlock.state.load(Ordering::Acquire) as u32;

    loop {
        let old = if cached == EXCLUSIVE { 0 } else { cached };
        let new = old + 1;

        assert_ne!(new, EXCLUSIVE, "overflow");

        match rwlock.state.compare_exchange_weak(cached as i32, new as i32, Ordering::Acquire, Ordering::Relaxed) {
            Ok(_) => return 0,
            Err(value) if value as u32 == EXCLUSIVE => return EBUSY,
            Err(value) => {
                cached = value as u32;
                core::hint::spin_loop();
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_trywrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    let rwlock: &pthread_rwlock_t = &*rwlock;

    match rwlock.state.compare_exchange(0, EXCLUSIVE as i32, Ordering::Acquire, Ordering::Relaxed) {
        Ok(_) => 0,
        Err(_) => EBUSY,
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_unlock(rwlock: *const pthread_rwlock_t) -> c_int {
    let rwlock: &pthread_rwlock_t = &*rwlock;

    let old = rwlock.state.swap(0, Ordering::Release) as u32;

    if old == EXCLUSIVE {
        rwlock.state.notify_all();
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_wrlock(rwlock: *mut pthread_rwlock_t) -> c_int {
    pthread_rwlock_timedwrlock(rwlock, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_destroy(_attr: *mut pthread_rwlockattr_t) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_getpshared(attr: *const pthread_rwlockattr_t, pshared: *mut c_int) -> c_int {
    core::ptr::write(pshared, (*attr).pshared);

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> c_int {
    core::ptr::write(attr, RwlockAttr {
        // Default according to POSIX.
        pshared: PTHREAD_PROCESS_PRIVATE,
    });

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_setpshared(attr: *mut pthread_rwlockattr_t, pshared: c_int) -> c_int {
    (*attr).pshared = pshared;
    0
}
