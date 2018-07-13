//! stdlib implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/stdlib.h.html

#![no_std]
#![feature(core_intrinsics)]

extern crate ctype;
extern crate errno;
extern crate platform;
extern crate rand;
extern crate string;
extern crate time;
extern crate unistd;
extern crate wchar;

use core::{ptr, str};
use rand::distributions::Alphanumeric;
use rand::prng::XorShiftRng;
use rand::rngs::JitterRng;
use rand::{Rng, SeedableRng};
use string::*;
use wchar::*;

use errno::*;
use platform::types::*;

mod sort;

pub const EXIT_FAILURE: c_int = 1;
pub const EXIT_SUCCESS: c_int = 0;
pub const RAND_MAX: c_int = 2147483647;

//Maximum number of bytes in a multibyte character for the current locale
pub const MB_CUR_MAX: c_int = 4;
//Maximum number of bytes in a multibyte characters for any locale
pub const MB_LEN_MAX: c_int = 4;

static mut ATEXIT_FUNCS: [Option<extern "C" fn()>; 32] = [None; 32];
static mut RNG: Option<XorShiftRng> = None;

#[no_mangle]
pub unsafe extern "C" fn a64l(s: *const c_char) -> c_long {
    if s.is_null() {
        return 0;
    }
    let mut l: c_long = 0;
    // a64l does not support more than 6 characters at once
    for x in 0..6 {
        let c = *s.offset(x);
        if c == 0 {
            // string is null terminated
            return l;
        }
        // ASCII to base64 conversion:
        let mut bits: c_long = if c < 58 {
            (c - 46) as c_long // ./0123456789
        } else if c < 91 {
            (c - 53) as c_long // A-Z
        } else {
            (c - 59) as c_long // a-z
        };
        bits <<= 6 * x;
        l |= bits;
    }
    return l;
}

#[no_mangle]
pub unsafe extern "C" fn abort() {
    use core::intrinsics;

    intrinsics::abort();
}

#[no_mangle]
pub extern "C" fn abs(i: c_int) -> c_int {
    if i < 0 {
        -i
    } else {
        i
    }
}

#[no_mangle]
pub unsafe extern "C" fn aligned_alloc(alignment: size_t, size: size_t) -> *mut c_void {
    if size % alignment != 0 {
        return ptr::null_mut();
    }

    memalign(alignment, size)
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
pub unsafe extern "C" fn atof(s: *const c_char) -> c_double {
    strtod(s, ptr::null_mut())
}

macro_rules! dec_num_from_ascii {
    ($s:expr, $t:ty) => {
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

unsafe extern "C" fn void_cmp(a: *const c_void, b: *const c_void) -> c_int {
    return *(a as *const i32) - *(b as *const i32) as c_int;
}

#[no_mangle]
pub unsafe extern "C" fn bsearch(
    key: *const c_void,
    base: *const c_void,
    nel: size_t,
    width: size_t,
    compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
) -> *mut c_void {
    let mut start = base;
    let mut len = nel;
    let cmp_fn = compar.unwrap_or(void_cmp);
    while len > 0 {
        let med = (start as size_t + (len >> 1) * width) as *const c_void;
        let diff = cmp_fn(key, med);
        if diff == 0 {
            return med as *mut c_void;
        } else if diff > 0 {
            start = (med as usize + width) as *const c_void;
            len -= 1;
        }
        len >>= 1;
    }
    ptr::null_mut()
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

// #[no_mangle]
pub extern "C" fn drand48() -> c_double {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn ecvt(
    value: c_double,
    ndigit: c_int,
    decpt: *mut c_int,
    sign: *mut c_int,
) -> *mut c_char {
    unimplemented!();
}

// #[no_mangle]
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

// #[no_mangle]
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
    platform::free(ptr);
}

// #[no_mangle]
pub extern "C" fn gcvt(value: c_double, ndigit: c_int, buf: *mut c_char) -> *mut c_char {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn getenv(name: *const c_char) -> *mut c_char {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn getsubopt(
    optionp: *mut *mut c_char,
    tokens: *const *mut c_char,
    valuep: *mut *mut c_char,
) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn grantpt(fildes: c_int) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn initstate(seec: c_uint, state: *mut c_char, size: size_t) -> *mut c_char {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn jrand48(xsubi: [c_ushort; 3]) -> c_long {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn l64a(value: c_long) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn labs(i: c_long) -> c_long {
    if i < 0 {
        -i
    } else {
        i
    }
}

// #[no_mangle]
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

// #[no_mangle]
pub extern "C" fn lrand48() -> c_long {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn malloc(size: size_t) -> *mut c_void {
    platform::alloc(size)
}

#[no_mangle]
pub unsafe extern "C" fn memalign(alignment: size_t, size: size_t) -> *mut c_void {
    platform::alloc_align(size, alignment)
}

#[no_mangle]
pub unsafe extern "C" fn mblen(s: *const c_char, n: size_t) -> c_int {
    let mut wc: wchar_t = 0;
    let mut state: mbstate_t = mbstate_t {};
    let result: usize = mbrtowc(&mut wc, s, n, &mut state);

    if result == -1isize as usize {
        return -1;
    }
    if result == -2isize as usize {
        return -1;
    }

    result as i32
}

#[no_mangle]
pub unsafe extern "C" fn mbstowcs(pwcs: *mut wchar_t, mut s: *const c_char, n: size_t) -> size_t {
    let mut state: mbstate_t = mbstate_t {};
    mbsrtowcs(pwcs, &mut s, n, &mut state)
}

#[no_mangle]
pub unsafe extern "C" fn mbtowc(pwc: *mut wchar_t, s: *const c_char, n: size_t) -> c_int {
    let mut state: mbstate_t = mbstate_t {};
    mbrtowc(pwc, s, n, &mut state) as c_int
}

#[no_mangle]
pub extern "C" fn mktemp(name: *mut c_char) -> *mut c_char {
    use core::iter;
    use core::mem;
    use core::slice;
    let len = unsafe { strlen(name) };
    if len < 6 {
        unsafe { platform::errno = errno::EINVAL };
        unsafe { *name = 0 };
        return name;
    }
    for i in len - 6..len {
        if unsafe { *name.offset(i as isize) } != b'X' as c_char {
            unsafe { platform::errno = errno::EINVAL };
            unsafe { *name = 0 };
            return name;
        }
    }

    let mut rng = JitterRng::new_with_timer(get_nstime);
    rng.test_timer();

    let mut retries = 100;
    loop {
        let mut char_iter = iter::repeat(()).map(|()| rng.sample(Alphanumeric)).take(6);
        unsafe {
            for (i, c) in char_iter.enumerate() {
                *name.offset(len as isize - i as isize - 1) = c as c_char
            }
        }

        unsafe {
            let mut st: stat = mem::uninitialized();
            if platform::stat(name, &mut st) != 0 {
                if platform::errno != ENOENT {
                    *name = 0;
                }
                return name;
            }
            mem::forget(st);
        }
        retries = retries - 1;
        if retries == 0 {
            break;
        }
    }
    unsafe { platform::errno = EEXIST };
    unsafe { *name = 0 };
    name
}

fn get_nstime() -> u64 {
    use core::mem;
    use time::constants::CLOCK_MONOTONIC;
    let mut ts: timespec = unsafe { mem::uninitialized() };
    platform::clock_gettime(CLOCK_MONOTONIC, &mut ts);
    unsafe { ts.tv_nsec as u64 }
}

// #[no_mangle]
pub extern "C" fn mkstemp(name: *mut c_char) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn mrand48() -> c_long {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn nrand48(xsubi: [c_ushort; 3]) -> c_long {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn ptsname(fildes: c_int) -> *mut c_char {
    unimplemented!();
}

// #[no_mangle]
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
    if let Some(comp) = compar {
        // XXX: check width too?  not specified
        if nel > 0 {
            // XXX: maybe try to do mergesort/timsort first and fallback to introsort if memory
            //      allocation fails?  not sure what is ideal
            sort::introsort(base as *mut c_char, nel, width, comp);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rand() -> c_int {
    match RNG {
        Some(ref mut rng) => rng.gen_range(0, RAND_MAX),
        None => {
            let mut rng = XorShiftRng::from_seed([1; 16]);
            let ret = rng.gen_range(0, RAND_MAX);
            RNG = Some(rng);
            ret
        }
    }
}

// #[no_mangle]
pub extern "C" fn rand_r(seed: *mut c_uint) -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn random() -> c_long {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    platform::realloc(ptr, size)
}

// #[no_mangle]
pub extern "C" fn realpath(file_name: *const c_char, resolved_name: *mut c_char) -> *mut c_char {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn seed48(seed16v: [c_ushort; 3]) -> c_ushort {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn setkey(key: *const c_char) {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn setstate(state: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn srand(seed: c_uint) {
    RNG = Some(XorShiftRng::from_seed([seed as u8; 16]));
}

// #[no_mangle]
pub extern "C" fn srand48(seed: c_long) {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn srandom(seed: c_uint) {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn strtod(s: *const c_char, endptr: *mut *mut c_char) -> c_double {
    // TODO: endptr

    use core::str::FromStr;

    let s_str = str::from_utf8_unchecked(platform::c_str(s));
    match f64::from_str(s_str) {
        Ok(ok) => ok as c_double,
        Err(_err) => {
            platform::errno = EINVAL;
            0.0
        }
    }
}

pub fn is_positive(ch: c_char) -> Option<(bool, isize)> {
    match ch {
        0 => None,
        ch if ch == b'+' as c_char => Some((true, 1)),
        ch if ch == b'-' as c_char => Some((false, 1)),
        _ => Some((true, 0)),
    }
}

pub fn detect_base(s: *const c_char) -> Option<(c_int, isize)> {
    let first = unsafe { *s } as u8;
    match first {
        0 => None,
        b'0' => {
            let second = unsafe { *s.offset(1) } as u8;
            if second == b'X' || second == b'x' {
                Some((16, 2))
            } else if second >= b'0' && second <= b'7' {
                Some((8, 1))
            } else {
                // in this case, the prefix (0) is going to be the number
                Some((8, 0))
            }
        }
        _ => Some((10, 0)),
    }
}

pub unsafe fn convert_octal(s: *const c_char) -> Option<(c_ulong, isize, bool)> {
    if *s != 0 && *s == b'0' as c_char {
        if let Some((val, idx, overflow)) = convert_integer(s.offset(1), 8) {
            Some((val, idx + 1, overflow))
        } else {
            // in case the prefix is not actually a prefix
            Some((0, 1, false))
        }
    } else {
        None
    }
}

pub unsafe fn convert_hex(s: *const c_char) -> Option<(c_ulong, isize, bool)> {
    if (*s != 0 && *s == b'0' as c_char)
        && (*s.offset(1) != 0 && (*s.offset(1) == b'x' as c_char || *s.offset(1) == b'X' as c_char))
    {
        convert_integer(s.offset(2), 16).map(|(val, idx, overflow)| (val, idx + 2, overflow))
    } else {
        None
    }
}

pub fn convert_integer(s: *const c_char, base: c_int) -> Option<(c_ulong, isize, bool)> {
    // -1 means the character is invalid
    #[cfg_attr(rustfmt, rustfmt_skip)]
    const LOOKUP_TABLE: [c_long; 256] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
         0,  1,  2,  3,  4,  5,  6,  7,  8,  9, -1, -1, -1, -1, -1, -1,
        -1, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, -1, -1, -1, -1, -1,
        -1, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    ];

    let mut num: c_ulong = 0;
    let mut idx = 0;
    let mut overflowed = false;

    loop {
        let val = unsafe { LOOKUP_TABLE[*s.offset(idx) as usize] };
        if val == -1 || val as c_int >= base {
            break;
        } else {
            if let Some(res) = num
                .checked_mul(base as c_ulong)
                .and_then(|num| num.checked_add(val as c_ulong))
            {
                num = res;
            } else {
                unsafe {
                    platform::errno = ERANGE;
                }
                num = c_ulong::max_value();
                overflowed = true;
            }

            idx += 1;
        }
    }

    if idx > 0 {
        Some((num, idx, overflowed))
    } else {
        None
    }
}

#[macro_export]
macro_rules! strto_impl {
    (
        $rettype:ty, $signed:expr, $maxval:expr, $minval:expr, $s:ident, $endptr:ident, $base:ident
    ) => {{
        // ensure these are constants
        const CHECK_SIGN: bool = $signed;
        const MAX_VAL: $rettype = $maxval;
        const MIN_VAL: $rettype = $minval;

        let set_endptr = |idx: isize| {
            if !$endptr.is_null() {
                // This is stupid, but apparently strto* functions want
                // const input but mut output, yet the man page says
                // "stores the address of the first invalid character in *endptr"
                // so obviously it doesn't want us to clone it.
                *$endptr = $s.offset(idx) as *mut _;
            }
        };

        let invalid_input = || {
            platform::errno = EINVAL;
            set_endptr(0);
        };

        // only valid bases are 2 through 36
        if $base != 0 && ($base < 2 || $base > 36) {
            invalid_input();
            return 0;
        }

        let mut idx = 0;

        // skip any whitespace at the beginning of the string
        while ctype::isspace(*$s.offset(idx) as c_int) != 0 {
            idx += 1;
        }

        // check for +/-
        let positive = match is_positive(*$s.offset(idx)) {
            Some((pos, i)) => {
                idx += i;
                pos
            }
            None => {
                invalid_input();
                return 0;
            }
        };

        // convert the string to a number
        let num_str = $s.offset(idx);
        let res = match $base {
            0 => detect_base(num_str)
                .and_then(|($base, i)| convert_integer(num_str.offset(i), $base)),
            8 => convert_octal(num_str),
            16 => convert_hex(num_str),
            _ => convert_integer(num_str, $base),
        };

        // check for error parsing octal/hex prefix
        // also check to ensure a number was indeed parsed
        let (num, i, overflow) = match res {
            Some(res) => res,
            None => {
                invalid_input();
                return 0;
            }
        };
        idx += i;

        let overflow = if CHECK_SIGN {
            overflow || (num as c_long).is_negative()
        } else {
            overflow
        };
        // account for the sign
        let num = num as $rettype;
        let num = if overflow {
            platform::errno = ERANGE;
            if CHECK_SIGN {
                if positive {
                    MAX_VAL
                } else {
                    MIN_VAL
                }
            } else {
                MAX_VAL
            }
        } else {
            if positive {
                num
            } else {
                // not using -num to keep the compiler happy
                num.overflowing_neg().0
            }
        };

        set_endptr(idx);

        num
    }};
}

#[no_mangle]
pub unsafe extern "C" fn strtoul(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> c_ulong {
    strto_impl!(
        c_ulong,
        false,
        c_ulong::max_value(),
        c_ulong::min_value(),
        s,
        endptr,
        base
    )
}

#[no_mangle]
pub unsafe extern "C" fn strtol(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_long {
    strto_impl!(
        c_long,
        true,
        c_long::max_value(),
        c_long::min_value(),
        s,
        endptr,
        base
    )
}

#[no_mangle]
pub unsafe extern "C" fn system(command: *const c_char) -> c_int {
    let child_pid = unistd::fork();
    if child_pid == 0 {
        let command_nonnull = if command.is_null() {
            "exit 0\0".as_ptr()
        } else {
            command as *const u8
        };

        let shell = "/bin/sh\0".as_ptr();

        let args = [
            "sh\0".as_ptr(),
            "-c\0".as_ptr(),
            command_nonnull,
            ptr::null()
        ];

        unistd::execv(shell as *const c_char, args.as_ptr() as *const *mut c_char);

        exit(127);

        unreachable!();
    } else {
        let mut wstatus = 0;
        if platform::waitpid(child_pid, &mut wstatus, 0) < 0 {
            return -1;
        }

        wstatus
    }
}

// #[no_mangle]
pub extern "C" fn ttyslot() -> c_int {
    unimplemented!();
}

// #[no_mangle]
pub extern "C" fn unlockpt(fildes: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn valloc(size: size_t) -> *mut c_void {
    memalign(4096, size)
}

#[no_mangle]
pub extern "C" fn wcstombs(s: *mut c_char, pwcs: *mut *const wchar_t, n: size_t) -> size_t {
    let mut state: mbstate_t = mbstate_t {};
    wcsrtombs(s, pwcs, n, &mut state)
}

#[no_mangle]
pub unsafe extern "C" fn wctomb(s: *mut c_char, wc: wchar_t) -> c_int {
    let mut state: mbstate_t = mbstate_t {};
    let result: usize = wcrtomb(s, wc, &mut state);

    if result == -1isize as usize {
        return -1;
    }
    if result == -2isize as usize {
        return -1;
    }

    result as c_int
}
