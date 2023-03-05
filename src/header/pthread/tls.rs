use super::*;

// TODO: Hashmap?
use alloc::collections::BTreeMap;

use core::cell::{Cell, RefCell};

use crate::header::errno::EINVAL;

// TODO: What should this limit be?
pub const PTHREAD_KEYS_MAX: u32 = 4096 * 32;

#[no_mangle]
pub unsafe extern "C" fn pthread_getspecific(key: pthread_key_t) -> *mut c_void {
    let Some(&Record { data, .. }) = DICT.borrow_mut().get(&key) else {
        return core::ptr::null_mut();
    };

    data
}
#[no_mangle]
pub unsafe extern "C" fn pthread_key_create(key_ptr: *mut pthread_key_t, destructor: extern "C" fn(value: *mut c_void)) -> c_int {
    let key = NEXTKEY.get();
    NEXTKEY.set(key + 1);

    // TODO
    //if key >= PTHREAD_KEYS_MAX {
    //}

    DICT.borrow_mut().insert(key, Record {
        data: core::ptr::null_mut(),
        destructor,
    });

    key_ptr.write(key);

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_key_delete(key: pthread_key_t) -> c_int {
    if DICT.borrow_mut().remove(&key).is_none() {
        // We don't have to return anything, but it's not less expensive to ignore it.
        return EINVAL;
    }

    0
}

 #[no_mangle]
pub unsafe extern "C" fn pthread_setspecific(key: pthread_key_t, value: *const c_void) -> c_int {
    let mut guard = DICT.borrow_mut();

    let Some(Record { data, .. }) = guard.get_mut(&key) else {
        // We don't have to return anything, but it's not less expensive to ignore it.
        return EINVAL;
    };

    *data = value as *mut c_void;

    0
}

#[thread_local]
static DICT: RefCell<BTreeMap<u32, Record>> = RefCell::new(BTreeMap::new());

struct Record {
    data: *mut c_void,
    destructor: extern "C" fn(value: *mut c_void),
}

#[thread_local]
static NEXTKEY: Cell<u32> = Cell::new(1);
