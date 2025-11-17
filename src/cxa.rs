use crate::platform::types::*;
use alloc::vec::Vec;
use core::ptr;
use spin::Mutex;

#[derive(Clone, Copy)]
struct CxaAtExitFunc {
    func: extern "C" fn(*mut c_void),
    arg: usize,
    dso: usize,
}

/// list of finalizer functions.
static CXA_ATEXIT_FUNCS: Mutex<Vec<Option<CxaAtExitFunc>>> = Mutex::new(Vec::new());

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __cxa_atexit(
    func: Option<extern "C" fn(*mut c_void)>,
    arg: *mut c_void,
    dso: *mut c_void,
) -> c_int {
    let Some(func) = func else {
        return 0;
    };

    let entry = CxaAtExitFunc {
        func,
        arg: arg as usize,
        dso: dso as usize,
    };

    let mut funcs = CXA_ATEXIT_FUNCS.lock();

    for slot in funcs.iter_mut() {
        if slot.is_none() {
            *slot = Some(entry);
            return 0;
        }
    }

    // No empty slots
    funcs.push(Some(entry));
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __cxa_finalize(dso: *mut c_void) {
    let mut funcs = CXA_ATEXIT_FUNCS.lock();

    let dso_usize = dso as usize;

    for slot in funcs.iter_mut().rev() {
        if let Some(entry) = slot.as_ref() {
            if dso.is_null() || entry.dso == dso_usize {
                if let Some(entry_to_run) = slot.take() {
                    (entry_to_run.func)(entry_to_run.arg as *mut c_void);
                }
            }
        }
    }

    // clean up remaining list
    if dso.is_null() {
        funcs.clear();
    } else {
        funcs.retain(|opt| opt.is_some());
    }
}
