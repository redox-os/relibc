use crate::platform::types::{c_int, c_void};
use alloc::vec::Vec;
use core::cell::RefCell;
use spin::Mutex;

#[derive(Clone, Copy)]
struct CxaAtExitFunc {
    func: extern "C" fn(*mut c_void),
    arg: usize,
    dso: usize,
}

#[derive(Clone, Copy)]
struct CxaThreadAtExitFunc {
    func: extern "C" fn(*mut c_void),
    obj: *mut c_void,
    dso: *mut c_void,
}

static CXA_ATEXIT_FUNCS: Mutex<Vec<Option<CxaAtExitFunc>>> = Mutex::new(Vec::new());
#[thread_local]
static DTORS: RefCell<Vec<CxaThreadAtExitFunc>> = RefCell::new(Vec::new());

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
    loop {
        let mut to_run: Vec<CxaAtExitFunc> = Vec::new();
        {
            let mut funcs = CXA_ATEXIT_FUNCS.lock();
            let dso_usize = dso as usize;

            for slot in funcs.iter_mut().rev() {
                if let Some(entry) = slot.as_ref()
                    && (dso.is_null() || entry.dso == dso_usize)
                    && let Some(entry_to_run) = slot.take()
                {
                    to_run.push(entry_to_run);
                }
            }
        }

        if to_run.is_empty() {
            break;
        }

        for entry in to_run {
            (entry.func)(entry.arg as *mut c_void);
        }
    }

    // clean up remaining list
    let mut funcs = CXA_ATEXIT_FUNCS.lock();
    if dso.is_null() {
        funcs.clear();
    } else {
        funcs.retain(Option::is_some);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __cxa_thread_atexit_impl(
    func: extern "C" fn(*mut c_void),
    obj: *mut c_void,
    dso: *mut c_void,
) {
    let entry = CxaThreadAtExitFunc { func, obj, dso };
    DTORS.borrow_mut().push(entry);
}

// called internally
pub unsafe fn __cxa_thread_finalize() {
    loop {
        // Take DTORS because they could call atexit() again
        let mut current_dtors = core::mem::take(&mut *DTORS.borrow_mut());

        if current_dtors.is_empty() {
            break;
        }

        while let Some(entry) = current_dtors.pop() {
            (entry.func)(entry.obj);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _ITM_deregisterTMCloneTable(_ptr: *mut c_void) {
    // No-op
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _ITM_registerTMCloneTable(_ptr: *mut c_void, _len: usize) {
    // No-op
}
