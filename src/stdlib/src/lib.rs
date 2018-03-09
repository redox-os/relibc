//! stdlib implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/stdlib.h.html

#![no_std]
#![feature(core_intrinsics)]
#![feature(global_allocator)]

extern crate ctype;
extern crate platform;
extern crate ralloc;

use platform::types::*;

#[global_allocator]
static ALLOCATOR: ralloc::Allocator = ralloc::Allocator;

pub const EXIT_FAILURE: c_int = 1;
pub const EXIT_SUCCESS: c_int = 0;

static mut ATEXIT_FUNCS: [Option<extern "C" fn()>; 32] = [None; 32];

#[no_mangle]
pub extern "C" fn a64l(s: *const c_char) -> c_long {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn abort() {
    use core::intrinsics;

    intrinsics::abort();
}

#[no_mangle]
pub extern "C" fn abs(i: c_int) -> c_int {
    if i < 0 { -i } else { i }
}

#[no_mangle]
pub unsafe extern "C" fn atexit(func: Option<extern "C" fn()>) -> c_int {
    for i in 0..ATEXIT_FUNCS.len() {
        if ATEXIT_FUNCS[i] == None {
            ATEXIT_FUNCS[i] = func;
            return 0;
        }
    }

    1
}

#[no_mangle]
pub extern "C" fn atof(s: *const c_char) -> c_double {
    unimplemented!();
}

macro_rules! dec_num_from_ascii {
    ($s: expr, $t: ty) => {
        unsafe {
            let mut s = $s;
            // Iterate past whitespace
            while ctype::isspace(*s as c_int) != 0 {
                s = s.offset(1);
            }

            // Find out if there is a - sign
            let neg_sign = match *s {
                0x2d => {
                    s = s.offset(1);
                    true
                }
                // '+' increment s and continue parsing
                0x2b => {
                    s = s.offset(1);
                    false
                }
                _ => false,
            };

            let mut n: $t = 0;
            while ctype::isdigit(*s as c_int) != 0 {
                n = 10 * n - (*s as $t - 0x30);
                s = s.offset(1);
            }

            if neg_sign {
                n
            } else {
                -n
            }
        }
    };
}

#[no_mangle]
pub extern "C" fn atoi(s: *const c_char) -> c_int {
    dec_num_from_ascii!(s, c_int)
}

#[no_mangle]
pub extern "C" fn atol(s: *const c_char) -> c_long {
    dec_num_from_ascii!(s, c_long)
}

#[no_mangle]
pub extern "C" fn bsearch(
    key: *const c_void,
    base: *const c_void,
    nel: size_t,
    width: size_t,
    compar: Option<extern "C" fn(*const c_void, *const c_void) -> c_int>,
) -> *mut c_void {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn calloc(nelem: size_t, elsize: size_t) -> *mut c_void {
    use core::intrinsics;

    let size = nelem * elsize;
    let ptr = malloc(size);
    if !ptr.is_null() {
        intrinsics::write_bytes(ptr as *mut u8, 0, size);
    }
    ptr
}

#[repr(C)]
pub struct div_t {
    quot: c_int,
    rem: c_int,
}

#[no_mangle]
pub extern "C" fn div(numer: c_int, denom: c_int) -> div_t {
    div_t {
        quot: numer / denom,
        rem: numer % denom,
    }
}

#[no_mangle]
pub extern "C" fn drand48() -> c_double {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ecvt(
    value: c_double,
    ndigit: c_int,
    decpt: *mut c_int,
    sign: *mut c_int,
) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn erand(xsubi: [c_ushort; 3]) -> c_double {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn exit(status: c_int) {
    for i in (0..ATEXIT_FUNCS.len()).rev() {
        if let Some(func) = ATEXIT_FUNCS[i] {
            (func)();
        }
    }

    platform::exit(status);
}

#[no_mangle]
pub extern "C" fn fcvt(
    value: c_double,
    ndigit: c_int,
    decpt: *mut c_int,
    sign: *mut c_int,
) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    let ptr = (ptr as *mut u8).offset(-16);
    let size = *(ptr as *mut u64);
    let _align = *(ptr as *mut u64).offset(1);
    ralloc::free(ptr, size as usize);
}

#[no_mangle]
pub extern "C" fn gcvt(value: c_double, ndigit: c_int, buf: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getenv(name: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getsubopt(
    optionp: *mut *mut c_char,
    tokens: *const *mut c_char,
    valuep: *mut *mut c_char,
) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn grantpt(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn initstate(seec: c_uint, state: *mut c_char, size: size_t) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn jrand48(xsubi: [c_ushort; 3]) -> c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn l64a(value: c_long) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn labs(i: c_long) -> c_long {
    if i < 0 { -i } else { i }
}

#[no_mangle]
pub extern "C" fn lcong48(param: [c_ushort; 7]) {
    unimplemented!();
}

#[repr(C)]
pub struct ldiv_t {
    quot: c_long,
    rem: c_long,
}

#[no_mangle]
pub extern "C" fn ldiv(numer: c_long, denom: c_long) -> ldiv_t {
    ldiv_t {
        quot: numer / denom,
        rem: numer % denom,
    }
}

#[no_mangle]
pub extern "C" fn lrand48() -> c_long {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn malloc(size: size_t) -> *mut c_void {
    let align = 8;
    let ptr = ralloc::alloc(size + 16, align);
    if !ptr.is_null() {
        *(ptr as *mut u64) = (size + 16) as u64;
        *(ptr as *mut u64).offset(1) = align as u64;
        ptr.offset(16) as *mut c_void
    } else {
        ptr as *mut c_void
    }
}

#[no_mangle]
pub extern "C" fn mblen(s: *const c_char, n: size_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mbstowcs(pwcs: *mut wchar_t, s: *const c_char, n: size_t) -> size_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mbtowc(pwc: *mut wchar_t, s: *const c_char, n: size_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mktemp(template: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mkstemp(template: *mut c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mrand48() -> c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn nrand48(xsubi: [c_ushort; 3]) -> c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ptsname(fildes: c_int) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putenv(s: *mut c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn qsort(
    base: *mut c_void,
    nel: size_t,
    width: size_t,
    compar: Option<extern "C" fn(*const c_void, *const c_void) -> c_int>,
) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rand() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rand_r(seed: *mut c_uint) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn random() -> c_long {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    let old_ptr = (ptr as *mut u8).offset(-16);
    let old_size = *(old_ptr as *mut u64);
    let align = *(old_ptr as *mut u64).offset(1);
    let ptr = ralloc::realloc(old_ptr, old_size as usize, size + 16, align as usize);
    if !ptr.is_null() {
        *(ptr as *mut u64) = (size + 16) as u64;
        *(ptr as *mut u64).offset(1) = align;
        ptr.offset(16) as *mut c_void
    } else {
        ptr as *mut c_void
    }
}

#[no_mangle]
pub extern "C" fn realpath(file_name: *const c_char, resolved_name: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn seed48(seed16v: [c_ushort; 3]) -> c_ushort {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setkey(key: *const c_char) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setstate(state: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn srand(seed: c_uint) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn srand48(seed: c_long) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn srandom(seed: c_uint) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strtod(s: *const c_char, endptr: *mut *mut c_char) -> c_double {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strtol(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn strtoul(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_ulong {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn system(command: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ttyslot() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn unlockpt(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn valloc(size: size_t) -> *mut c_void {
    let align = 4096;
    let ptr = ralloc::alloc(size + 16, align);
    if !ptr.is_null() {
        *(ptr as *mut u64) = (size + 16) as u64;
        *(ptr as *mut u64).offset(1) = align as u64;
        ptr.offset(16) as *mut c_void
    } else {
        ptr as *mut c_void
    }
}

#[no_mangle]
pub extern "C" fn wcstombs(s: *mut c_char, pwcs: *const wchar_t, n: size_t) -> size_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn wctomb(s: *mut c_char, wchar: wchar_t) -> c_int {
    unimplemented!();
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
