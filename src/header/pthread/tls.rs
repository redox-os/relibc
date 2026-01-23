// FIXME(andypython): remove this when #![allow(warnings, unused_variables)] is
// dropped from src/lib.rs.
#![warn(warnings, unused_variables)]

use super::*;

// TODO: Hashmap?
use alloc::{collections::BTreeMap, vec::Vec};

use core::{
    cell::RefCell,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    header::{errno::EINVAL, limits::PTHREAD_DESTRUCTOR_ITERATIONS},
    sync::Mutex,
};

type Dtor = Option<extern "C" fn(value: *mut c_void)>;

struct Record {
    data: *mut c_void,
}

#[thread_local]
static VALUES: RefCell<BTreeMap<pthread_key_t, Record>> = RefCell::new(BTreeMap::new());
static KEYS: Mutex<BTreeMap<pthread_key_t, Dtor>> = Mutex::new(BTreeMap::new());
static NEXTKEY: AtomicUsize = AtomicUsize::new(1);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_getspecific(key: pthread_key_t) -> *mut c_void {
    // According to POSIX (issue 8): Calling [`pthread_getspecific`] with a key
    // that has been deleted with [`pthread_key_delete`] or not obtained from
    // [`pthread_key_create`] results in undefined behaviour. Therefore, we only
    // do this check when debug assertions are explicitly enabled to avoid
    // acquiring the global [`KEYS`] lock when it is not necessary.
    debug_assert!(KEYS.lock().contains_key(&key));
    VALUES
        .borrow_mut()
        .get(&key)
        .map(|record| record.data)
        .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_setspecific(key: pthread_key_t, value: *const c_void) -> c_int {
    if !KEYS.lock().contains_key(&key) {
        // We don't have to return anything, but it's not less expensive to ignore it.
        //println!("Invalid key for pthread_setspecific key {:#0x} value {:p}", key, value);
        return EINVAL;
    }

    let mut guard = VALUES.borrow_mut();

    let record = guard.entry(key).or_insert(Record {
        data: core::ptr::null_mut(),
    });
    //println!("Valid key for pthread_setspecific key {:#0x} value {:p} (was {:p})", key, value, record.data);

    record.data = value as *mut c_void;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_key_create(
    key_ptr: *mut pthread_key_t,
    destructor: Dtor,
) -> c_int {
    let key = NEXTKEY.fetch_add(1, Ordering::SeqCst) as pthread_key_t;

    // TODO
    //if key >= PTHREAD_KEYS_MAX {
    //}

    KEYS.lock().insert(key, destructor);

    unsafe { key_ptr.write(key) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_key_delete(key: pthread_key_t) -> c_int {
    if KEYS.lock().remove(&key).is_none() || VALUES.borrow_mut().remove(&key).is_none() {
        // We don't have to return anything, but it's not less expensive to ignore it.
        return EINVAL;
    }

    0
}

pub(crate) unsafe fn run_all_destructors() {
    for _ in 0..PTHREAD_DESTRUCTOR_ITERATIONS {
        let mut any_run = false;
        let dtors = {
            let keys = KEYS.lock();
            keys.iter()
                .filter_map(|(&key, &dtor)| dtor.map(|dtor| (key, dtor)))
                .collect::<Vec<_>>()
        };

        // According to POSIX (issue 8): There is no specific order in which we
        // have to run the destructors.
        for (key, dtor) in dtors {
            let mut values = VALUES.borrow_mut();
            if let Some(record) = values.get_mut(&key) {
                let val = record.data;
                if val.is_null() {
                    continue;
                }
                record.data = ptr::null_mut();
                drop(values);
                dtor(val);
                any_run = true;
            }
        }

        if !any_run {
            break;
        }
    }

    // According to POSIX (issue 8): If even after
    // [`PTHREAD_DESTRUCTOR_ITERATIONS`] iterations there are still some
    // non-NULL values with associated destructors, the behaviour is
    // implementation-defined. We can choose to stop calling them or continue
    // calling them until none are left. Both musl and glibc choose to stop
    // calling them so we do the same.
}
