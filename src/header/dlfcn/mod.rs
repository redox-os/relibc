//! dlfcn implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/dlfcn.h.html

use core::{
    ptr, str,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{c_str::CStr, ld_so::tcb::Tcb, platform::types::*};

pub const RTLD_LAZY: c_int = 0x0001;
pub const RTLD_NOW: c_int = 0x0002;
pub const RTLD_GLOBAL: c_int = 0x0100;
pub const RTLD_LOCAL: c_int = 0x0000;

static ERROR_NOT_SUPPORTED: &'static CStr = c_str!("dlfcn not supported");

#[thread_local]
static ERROR: AtomicUsize = AtomicUsize::new(0);

#[repr(C)]
pub struct Dl_info {
    dli_fname: *const c_char,
    dli_fbase: *mut c_void,
    dli_sname: *const c_char,
    dli_saddr: *mut c_void,
}

#[no_mangle]
pub unsafe extern "C" fn dladdr(addr: *mut c_void, info: *mut Dl_info) -> c_int {
    (*info).dli_fname = ptr::null();
    (*info).dli_fbase = ptr::null_mut();
    (*info).dli_sname = ptr::null();
    (*info).dli_saddr = ptr::null_mut();
    0
}

#[no_mangle]
pub unsafe extern "C" fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void {
    let filename_opt = if filename.is_null() {
        None
    } else {
        Some(str::from_utf8_unchecked(
            CStr::from_ptr(filename).to_bytes(),
        ))
    };

    eprintln!("dlopen({:?}, {:#>04x})", filename_opt, flags);

    if let Some(filename) = filename_opt {
        if let Some(tcb) = Tcb::current() {
            if tcb.linker_ptr.is_null() {
                eprintln!("dlopen: linker not found");
                ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
                return ptr::null_mut();
            }

            eprintln!("dlopen: linker_ptr: {:p}", tcb.linker_ptr);
            let mut linker = (&*tcb.linker_ptr).lock();

            if let Err(err) = linker.load_library(filename) {
                eprintln!("dlopen: failed to load {}", filename);
                ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
                return ptr::null_mut();
            }

            if let Err(err) = linker.link(None, None) {
                eprintln!("dlopen: failed to link '{}': {}", filename, err);
                ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
                return ptr::null_mut();
            };

            // TODO
            1 as *mut c_void
        } else {
            eprintln!("dlopen: tcb not found");
            ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
            ptr::null_mut()
        }
    } else {
        1 as *mut c_void
    }
}

#[no_mangle]
pub unsafe extern "C" fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
    if symbol.is_null() {
        ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
        return ptr::null_mut();
    }

    let symbol_str = str::from_utf8_unchecked(CStr::from_ptr(symbol).to_bytes());

    eprintln!("dlsym({:p}, {})", handle, symbol_str);

    if let Some(tcb) = Tcb::current() {
        if tcb.linker_ptr.is_null() {
            eprintln!("dlopen: linker not found");
            ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
            return ptr::null_mut();
        }

        eprintln!("dlsym: linker_ptr: {:p}", tcb.linker_ptr);
        let linker = (&*tcb.linker_ptr).lock();

        if let Some(global) = linker.get_sym(symbol_str) {
            eprintln!("dlsym({:p}, {}) = 0x{:x}", handle, symbol_str, global);
            global as *mut c_void
        } else {
            eprintln!("dlsym: symbol not found");
            ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
            ptr::null_mut()
        }
    } else {
        eprintln!("dlsym: tcb not found");
        ERROR.store(ERROR_NOT_SUPPORTED.as_ptr() as usize, Ordering::SeqCst);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn dlclose(handle: *mut c_void) -> c_int {
    // TODO: Loader::fini() should be called about here

    0
}

#[no_mangle]
pub extern "C" fn dlerror() -> *mut c_char {
    ERROR.swap(0, Ordering::SeqCst) as *mut c_char
}
