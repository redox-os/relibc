//! `dlfcn.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/dlfcn.h.html>.

#![deny(unsafe_op_in_unsafe_fn)]
// FIXME(andypython): remove this when #![allow(warnings, unused_variables)] is
// dropped from src/lib.rs.
#![warn(warnings, unused_variables)]

use core::{
    ptr, str,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    c_str::CStr,
    ld_so::{
        linker::{DlError, ObjectHandle, Resolve, ScopeKind},
        tcb::Tcb,
    },
    platform::types::*,
};

pub const RTLD_LAZY: c_int = 1 << 0;
pub const RTLD_NOW: c_int = 1 << 1;
pub const RTLD_NOLOAD: c_int = 1 << 2;
pub const RTLD_GLOBAL: c_int = 1 << 8;
pub const RTLD_LOCAL: c_int = 0x0000;

pub const RTLD_DEFAULT: *mut c_void = 0 as *mut c_void; // XXX: cbindgen doesn't like ptr::null_mut()

static ERROR_NOT_SUPPORTED: &core::ffi::CStr = c"dlfcn not supported";

#[thread_local]
static ERROR: AtomicUsize = AtomicUsize::new(0);

fn set_last_error(error: DlError) {
    ERROR.store(error.repr().as_ptr() as usize, Ordering::SeqCst);
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/dlfcn.h.html>.
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct Dl_info {
    dli_fname: *const c_char,
    dli_fbase: *mut c_void,
    dli_sname: *const c_char,
    dli_saddr: *mut c_void,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dladdr.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dladdr(_addr: *mut c_void, info: *mut Dl_info) -> c_int {
    //TODO
    unsafe {
        (*info).dli_fname = ptr::null();
        (*info).dli_fbase = ptr::null_mut();
        (*info).dli_sname = ptr::null();
        (*info).dli_saddr = ptr::null_mut();
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dlopen.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlopen(cfilename: *const c_char, flags: c_int) -> *mut c_void {
    //TODO support all sort of flags
    let resolve = if flags & RTLD_NOW == RTLD_NOW {
        Resolve::Now
    } else {
        Resolve::Lazy
    };

    let scope = if flags & RTLD_GLOBAL == RTLD_GLOBAL {
        ScopeKind::Global
    } else {
        ScopeKind::Local
    };

    let noload = flags & RTLD_NOLOAD == RTLD_NOLOAD;

    let filename = if cfilename.is_null() {
        None
    } else {
        unsafe {
            Some(str::from_utf8_unchecked(
                CStr::from_ptr(cfilename).to_bytes(),
            ))
        }
    };

    let tcb = match unsafe { Tcb::current() } {
        Some(tcb) => tcb,
        None => {
            ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
            return ptr::null_mut();
        }
    };

    if tcb.linker_ptr.is_null() {
        ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
        return ptr::null_mut();
    }

    let mut linker = unsafe { (*tcb.linker_ptr).lock() };

    let cbs_c = linker.cbs.clone();
    let cbs = cbs_c.borrow();

    match (cbs.load_library)(&mut linker, filename, resolve, scope, noload) {
        Ok(handle) => handle.as_ptr().cast_mut(),
        Err(error) => {
            set_last_error(error);
            ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dlsym.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
    let handle = ObjectHandle::from_ptr(handle);

    if symbol.is_null() {
        ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
        return ptr::null_mut();
    }

    let symbol_str = unsafe { str::from_utf8_unchecked(CStr::from_ptr(symbol).to_bytes()) };

    // FIXME(andypython): just call obj.scope.get_sym() directly or search the
    // global scope.  The rest is unnecessary as Linker::get_sym() does not
    // depend on the Linker state.
    let tcb = match unsafe { Tcb::current() } {
        Some(tcb) => tcb,
        None => {
            ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
            return ptr::null_mut();
        }
    };

    if tcb.linker_ptr.is_null() {
        ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
        return ptr::null_mut();
    }

    let linker = unsafe { (*tcb.linker_ptr).lock() };
    let cbs_c = linker.cbs.clone();
    let cbs = cbs_c.borrow();
    match (cbs.get_sym)(&linker, handle, symbol_str) {
        Some(sym) => sym,
        _ => {
            ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
            ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dlclose.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dlclose(handle: *mut c_void) -> c_int {
    let tcb = match unsafe { Tcb::current() } {
        Some(tcb) => tcb,
        None => {
            ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
            return -1;
        }
    };

    if tcb.linker_ptr.is_null() {
        ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
        return -1;
    };

    let Some(handle) = ObjectHandle::from_ptr(handle) else {
        set_last_error(DlError::InvalidHandle);
        return -1;
    };

    let mut linker = unsafe { (*tcb.linker_ptr).lock() };
    let cbs_c = linker.cbs.clone();
    let cbs = cbs_c.borrow();
    (cbs.unload)(&mut linker, handle);
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/dlerror.html>.
#[unsafe(no_mangle)]
pub extern "C" fn dlerror() -> *mut c_char {
    ERROR.swap(0, Ordering::SeqCst) as *mut c_char
}
