use super::*;

// TODO: Hashmap?
use alloc::collections::BTreeMap;

use core::cell::{Cell, RefCell};

use crate::{header::errno::EINVAL, sync::Mutex};

// TODO: What should this limit be?
pub const PTHREAD_KEYS_MAX: u32 = 4096 * 32;

#[no_mangle]
pub unsafe extern "C" fn pthread_getspecific(key: pthread_key_t) -> *mut c_void {
    //print!("pthread_getspecific key={:#0x}: ", key);

    // TODO: Right?
    if !KEYS.lock().contains_key(&key) {
        //println!("= not found");
        return core::ptr::null_mut();
    }

    let Some(&Record { data, .. }) = VALUES.borrow_mut().get(&key) else {
        //println!("= NULL");
        return core::ptr::null_mut();
    };
    //println!("= {:p}", data);

    data
}
#[no_mangle]
pub unsafe extern "C" fn pthread_key_create(
    key_ptr: *mut pthread_key_t,
    destructor: Option<extern "C" fn(value: *mut c_void)>,
) -> c_int {
    let key = NEXTKEY.get();
    NEXTKEY.set(key + 1);
    //println!("pthread_key_create new key {:#0x}, dtor {:p}", key, destructor);

    // TODO
    //if key >= PTHREAD_KEYS_MAX {
    //}

    KEYS.lock().insert(key, Dtor { destructor });

    VALUES.borrow_mut().insert(
        key,
        Record {
            data: core::ptr::null_mut(),
        },
    );

    key_ptr.write(key);

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_key_delete(key: pthread_key_t) -> c_int {
    if KEYS.lock().remove(&key).is_none() || VALUES.borrow_mut().remove(&key).is_none() {
        // We don't have to return anything, but it's not less expensive to ignore it.
        return EINVAL;
    }
    //println!("pthread_key_delete {:#0x}", key);

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_setspecific(key: pthread_key_t, value: *const c_void) -> c_int {
    if !KEYS.lock().contains_key(&key) {
        // We don't have to return anything, but it's not less expensive to ignore it.
        //println!("Invalid key for pthread_setspecific key {:#0x} value {:p}", key, value);
        return EINVAL;
    }

    let mut guard = VALUES.borrow_mut();

    let Record { ref mut data, .. } = guard.entry(key).or_insert(Record {
        data: core::ptr::null_mut(),
    });
    //println!("Valid key for pthread_setspecific key {:#0x} value {:p} (was {:p})", key, value, *data);

    *data = value as *mut c_void;

    0
}

static KEYS: Mutex<BTreeMap<pthread_key_t, Dtor>> = Mutex::new(BTreeMap::new());

struct Dtor {
    destructor: Option<extern "C" fn(value: *mut c_void)>,
}

#[thread_local]
static VALUES: RefCell<BTreeMap<pthread_key_t, Record>> = RefCell::new(BTreeMap::new());

struct Record {
    data: *mut c_void,
}

#[thread_local]
static NEXTKEY: Cell<pthread_key_t> = Cell::new(1);

pub(crate) unsafe fn run_all_destructors() {
    for (key, Record { data }) in VALUES.take() {
        let Some(&Dtor { destructor: Some(dtor) }) = KEYS.lock().get(&key) else { continue };

        dtor(data);
    }
}
