//! stdlib implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/stdlib.h.html

#![no_std]
#![feature(global_allocator)]

extern crate platform;
extern crate ralloc;

use platform::types::*;

#[global_allocator]
static ALLOCATOR: ralloc::Allocator = ralloc::Allocator;

pub const EXIT_FAILURE: c_int = 1;
pub const EXIT_SUCCESS: c_int = 0;

static mut ATEXIT_FUNCS: [usize; 32] = [0; 32];

#[no_mangle]
pub unsafe extern "C" fn atexit(func: extern "C" fn()) -> c_int {
    for i in 0..ATEXIT_FUNCS.len() {
        if ATEXIT_FUNCS[i] == 0 {
            ATEXIT_FUNCS[i] = func as usize;
            return 0;
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn exit(status: c_int) {
    use core::mem;

    for i in (0..ATEXIT_FUNCS.len()).rev() {
        if ATEXIT_FUNCS[i] != 0 {
            let func = mem::transmute::<usize, extern "C" fn()>(ATEXIT_FUNCS[i]);
            (func)();
        }
    }

    platform::exit(status);
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    let ptr = (ptr as *mut u8).offset(-8);
    let size = *(ptr as *mut u64);
    ralloc::free(ptr, size as usize);
}

#[no_mangle]
pub unsafe extern "C" fn malloc(size: size_t) -> *mut c_void {
    let ptr = ralloc::alloc(size + 8, 8);
    *(ptr as *mut u64) = size as u64;
    ptr.offset(8) as *mut c_void
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
