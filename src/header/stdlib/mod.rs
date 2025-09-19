//! `stdlib.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdlib.h.html>.

use core::{convert::TryFrom, intrinsics, iter, mem, ptr, slice};
use rand::{
    distributions::{Alphanumeric, Distribution, Uniform},
    Rng, SeedableRng,
};
use rand_jitter::JitterRng;
use rand_xorshift::XorShiftRng;

use crate::{
    c_str::CStr,
    error::{Errno, ResultExt},
    fs::File,
    header::{
        ctype,
        errno::{self, *},
        fcntl::*,
        limits,
        stdio::flush_io_streams,
        string::*,
        sys_ioctl::*,
        time::constants::CLOCK_MONOTONIC,
        unistd::{self, sysconf, _SC_PAGESIZE},
        wchar::*,
    },
    ld_so,
    out::Out,
    platform::{self, types::*, Pal, Sys},
    sync::Once,
};

mod rand48;
mod random;
mod sort;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdlib.h.html>.
pub const EXIT_FAILURE: c_int = 1;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdlib.h.html>.
pub const EXIT_SUCCESS: c_int = 0;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdlib.h.html>.
pub const RAND_MAX: c_int = 2_147_483_647;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/stdlib.h.html>.
//Maximum number of bytes in a multibyte character for the current locale
pub const MB_CUR_MAX: c_int = 4;
/// Actually specified for `limits.h`?
//Maximum number of bytes in a multibyte characters for any locale
pub const MB_LEN_MAX: c_int = 4;

static mut ATEXIT_FUNCS: [Option<extern "C" fn()>; 32] = [None; 32];
static mut AT_QUICK_EXIT_FUNCS: [Option<extern "C" fn()>; 32] = [None; 32];
static mut L64A_BUFFER: [c_char; 7] = [0; 7]; // up to 6 digits plus null terminator
static mut RNG: Option<XorShiftRng> = None;

// TODO: This could be const fn, but the trait system won't allow that.
static RNG_SAMPLER: Once<Uniform<c_int>> = Once::new();

fn rng_sampler() -> &'static Uniform<c_int> {
    RNG_SAMPLER.call_once(|| Uniform::new_inclusive(0, RAND_MAX))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/_Exit.html>.
#[no_mangle]
pub extern "C" fn _Exit(status: c_int) -> ! {
    unistd::_exit(status);
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/a64l.html>.
#[no_mangle]
pub unsafe extern "C" fn a64l(s: *const c_char) -> c_long {
    // Early return upon null pointer argument
    if s.is_null() {
        return 0;
    }

    // POSIX says only the low-order 32 bits are used.
    let mut l: i32 = 0;

    // Handle up to 6 input characters (excl. null terminator)
    for i in 0..6 {
        let digit_char = *s.offset(i);

        let digit_value = match digit_char {
            0 => break, // Null terminator encountered
            46..=57 => {
                // ./0123456789 represents values 0 to 11. b'.' == 46
                digit_char - 46
            }
            65..=90 => {
                // A-Z for values 12 to 37. b'A' == 65, 65-12 == 53
                digit_char - 53
            }
            97..=122 => {
                // a-z for values 38 to 63. b'a' == 97, 97-38 == 59
                digit_char - 59
            }
            _ => return 0, // Early return for anything else
        };

        l |= i32::from(digit_value) << 6 * i;
    }

    c_long::from(l)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/abort.html>.
#[no_mangle]
pub unsafe extern "C" fn abort() -> ! {
    eprintln!("Abort");
    intrinsics::abort();
}

#[cfg(not(target_pointer_width = "64"))]
#[no_mangle]
static __stack_chk_guard: uintptr_t = 0x19fcadfe;

#[cfg(target_pointer_width = "64")]
#[no_mangle]
static __stack_chk_guard: uintptr_t = 0xd048c37519fcadfe;

#[no_mangle]
unsafe extern "C" fn __stack_chk_fail() -> ! {
    abort();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/abs.html>.
#[no_mangle]
pub extern "C" fn abs(i: c_int) -> c_int {
    i.abs()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/aligned_alloc.html>.
#[no_mangle]
pub unsafe extern "C" fn aligned_alloc(alignment: size_t, size: size_t) -> *mut c_void {
    if size % alignment == 0 {
        /* The size-is-multiple-of-alignment requirement is the only
         * difference between aligned_alloc() and memalign(). */
        memalign(alignment, size)
    } else {
        platform::ERRNO.set(EINVAL);
        ptr::null_mut()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/at_quick_exit.html>.
#[no_mangle]
pub unsafe extern "C" fn at_quick_exit(func: Option<extern "C" fn()>) -> c_int {
    for i in 0..AT_QUICK_EXIT_FUNCS.len() {
        if AT_QUICK_EXIT_FUNCS[i] == None {
            AT_QUICK_EXIT_FUNCS[i] = func;
            return 0;
        }
    }

    1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atexit.html>.
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atof.html>.
#[no_mangle]
pub unsafe extern "C" fn atof(s: *const c_char) -> c_double {
    strtod(s, ptr::null_mut())
}

macro_rules! dec_num_from_ascii {
    ($s:expr, $t:ty) => {{
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
    }};
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atoi.html>.
#[no_mangle]
pub unsafe extern "C" fn atoi(s: *const c_char) -> c_int {
    dec_num_from_ascii!(s, c_int)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atol.html>.
#[no_mangle]
pub unsafe extern "C" fn atol(s: *const c_char) -> c_long {
    dec_num_from_ascii!(s, c_long)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atol.html>.
#[no_mangle]
pub unsafe extern "C" fn atoll(s: *const c_char) -> c_longlong {
    dec_num_from_ascii!(s, c_longlong)
}

unsafe extern "C" fn void_cmp(a: *const c_void, b: *const c_void) -> c_int {
    *(a as *const i32) - *(b as *const i32) as c_int
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/bsearch.html>.
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/calloc.html>.
#[no_mangle]
pub unsafe extern "C" fn calloc(nelem: size_t, elsize: size_t) -> *mut c_void {
    //Handle possible integer overflow in size calculation
    match nelem.checked_mul(elsize) {
        Some(size) => {
            /* If allocation fails here, errno setting will be handled
             * by malloc() */
            let ptr = malloc(size);
            if !ptr.is_null() {
                ptr.write_bytes(0, size);
            }
            ptr
        }
        None => {
            // For overflowing multiplication, we have to set errno here
            platform::ERRNO.set(ENOMEM);
            ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/div.html>.
#[repr(C)]
pub struct div_t {
    quot: c_int,
    rem: c_int,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/div.html>.
#[no_mangle]
pub extern "C" fn div(numer: c_int, denom: c_int) -> div_t {
    div_t {
        quot: numer / denom,
        rem: numer % denom,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/drand48.html>.
///
/// # Panics
/// Panics if the function is unable to obtain a lock on the generator's global
/// state.
#[no_mangle]
pub extern "C" fn drand48() -> c_double {
    let params = rand48::params();
    let mut xsubi = rand48::xsubi_lock();
    *xsubi = params.step(*xsubi);
    xsubi.get_f64()
}

/// See <https://pubs.opengroup.org/onlinepubs/009695399/functions/ecvt.html>.
///
/// # Deprecation
/// The `ecvt()` function was marked as legacy in the Open Group Base
/// Specifications Issue 6, and the function was removed in Issue 7.
#[deprecated]
// #[no_mangle]
pub extern "C" fn ecvt(
    value: c_double,
    ndigit: c_int,
    decpt: *mut c_int,
    sign: *mut c_int,
) -> *mut c_char {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/drand48.html>.
///
/// # Safety
/// The caller must ensure that `xsubi` is convertible to a
/// `&mut [c_ushort; 3]`.
///
/// # Panics
/// Panics if the function is unable to obtain a lock on the generator's global
/// state.
#[no_mangle]
pub unsafe extern "C" fn erand48(xsubi: *mut c_ushort) -> c_double {
    let params = rand48::params();
    let xsubi_mut: &mut [c_ushort; 3] = slice::from_raw_parts_mut(xsubi, 3).try_into().unwrap();
    let new_xsubi_value = params.step(xsubi_mut.into());
    *xsubi_mut = new_xsubi_value.into();
    new_xsubi_value.get_f64()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exit.html>.
#[no_mangle]
pub unsafe extern "C" fn exit(status: c_int) -> ! {
    extern "C" {
        static __fini_array_start: extern "C" fn();
        static __fini_array_end: extern "C" fn();

        fn _fini();
    }

    for i in (0..ATEXIT_FUNCS.len()).rev() {
        if let Some(func) = ATEXIT_FUNCS[i] {
            (func)();
        }
    }

    // Look for the neighbor functions in memory until the end
    let mut f = &__fini_array_end as *const _;
    #[allow(clippy::op_ref)]
    while f > &__fini_array_start {
        f = f.offset(-1);
        (*f)();
    }

    #[cfg(not(target_arch = "riscv64"))] // risc-v uses arrays exclusively
    {
        _fini();
    }

    ld_so::fini();

    crate::pthread::terminate_from_main_thread();

    flush_io_streams();

    Sys::exit(status);
}

/// See <https://pubs.opengroup.org/onlinepubs/009695399/functions/ecvt.html>.
///
/// # Deprecation
/// The `fcvt()` function was marked as legacy in the Open Group Base
/// Specifications Issue 6, and the function was removed in Issue 7.
#[deprecated]
// #[no_mangle]
pub extern "C" fn fcvt(
    value: c_double,
    ndigit: c_int,
    decpt: *mut c_int,
    sign: *mut c_int,
) -> *mut c_char {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/free.html>.
#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    platform::free(ptr);
}

/// See <https://pubs.opengroup.org/onlinepubs/009695399/functions/ecvt.html>.
///
/// # Deprecation
/// The `gcvt()` function was marked as legacy in the Open Group Base
/// Specifications Issue 6, and the function was removed in Issue 7.
#[deprecated]
// #[no_mangle]
pub extern "C" fn gcvt(value: c_double, ndigit: c_int, buf: *mut c_char) -> *mut c_char {
    unimplemented!();
}

unsafe fn find_env(search: *const c_char) -> Option<(usize, *mut c_char)> {
    for (i, mut item) in platform::environ_iter().enumerate() {
        let mut search = search;
        loop {
            let end_of_query = *search == 0 || *search == b'=' as c_char;
            assert_ne!(*item, 0, "environ has an item without value");
            if *item == b'=' as c_char || end_of_query {
                if *item == b'=' as c_char && end_of_query {
                    // Both keys env here
                    return Some((i, item.add(1)));
                } else {
                    break;
                }
            }

            if *item != *search {
                break;
            }

            item = item.add(1);
            search = search.add(1);
        }
    }

    None
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getenv.html>.
#[no_mangle]
pub unsafe extern "C" fn getenv(name: *const c_char) -> *mut c_char {
    find_env(name).map(|val| val.1).unwrap_or(ptr::null_mut())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getsubopt.html>.
#[no_mangle]
pub unsafe extern "C" fn getsubopt(
    optionp: *mut *mut c_char,
    tokens: *const *mut c_char,
    valuep: *mut *mut c_char,
) -> c_int {
    let start = *optionp;
    let max = strlen(start) as isize;
    let mut i: usize = 0;
    *valuep = ptr::null_mut();
    *optionp = strchr(start, b',' as i32);
    if !(*optionp).is_null() {
        *(*optionp).add(0) = 0;
        *optionp = *optionp.add(1);
    } else {
        *(optionp) = start.add(max as usize);
    }
    while !tokens.offset(i as isize).is_null() {
        let cur = tokens.offset(i as isize) as *const _;
        let len = strlen(cur) as isize;
        if strncmp(cur, start, len as usize) != 0 {
            i = i + 1;
            continue;
        }
        if (*start.offset(len)) == b'=' as c_char {
            *valuep = start.offset(len + 1);
        } else if !start.offset(len).is_null() {
            i = i + 1;
            continue;
        }
        return i as c_int;
    }
    -1
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/grantpt.html>.
#[no_mangle]
pub extern "C" fn grantpt(fildes: c_int) -> c_int {
    // No-op on Linux and Redox
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/initstate.html>.
// Ported from musl
#[no_mangle]
pub unsafe extern "C" fn initstate(seed: c_uint, state: *mut c_char, size: size_t) -> *mut c_char {
    if size < 8 {
        ptr::null_mut()
    } else {
        let mut random_state = random::state_lock();
        let old_state = random_state.save();
        random_state.n = match size {
            0..=7 => unreachable!(), // ensured above
            8..=31 => 0,
            32..=63 => 7,
            64..=127 => 15,
            128..=255 => 31,
            _ => 63,
        };

        random_state.x_ptr = (state.cast::<[u8; 4]>()).offset(1);
        random_state.seed(seed);
        random_state.save();

        old_state.cast::<_>()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/drand48.html>.
///
/// # Safety
/// The caller must ensure that `xsubi` is convertible to a
/// `&mut [c_ushort; 3]`.
///
/// # Panics
/// Panics if the function is unable to obtain a lock on the generator's global
/// state.
#[no_mangle]
pub unsafe extern "C" fn jrand48(xsubi: *mut c_ushort) -> c_long {
    let params = rand48::params();
    let xsubi_mut: &mut [c_ushort; 3] = slice::from_raw_parts_mut(xsubi, 3).try_into().unwrap();
    let new_xsubi_value = params.step(xsubi_mut.into());
    *xsubi_mut = new_xsubi_value.into();
    new_xsubi_value.get_i32()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/a64l.html>.
#[no_mangle]
pub unsafe extern "C" fn l64a(value: c_long) -> *mut c_char {
    // POSIX says we should only consider the lower 32 bits of value.
    let value_as_i32 = value as i32;

    /* If we pretend to extend the 32-bit value with 4 binary zeros, we
     * would get a 36-bit integer. The number of base-64 digits to be
     * left unused can then be found by taking the number of leading
     * zeros, dividing by 6 and rounding down (i.e. using integer
     * division). */
    let num_output_digits = usize::try_from(6 - (value_as_i32.leading_zeros() + 4) / 6).unwrap();

    // Reset buffer (and have null terminator in place for any result)
    L64A_BUFFER = [0; 7];

    for i in 0..num_output_digits {
        // Conversion to c_char always succeeds for the range 0..=63
        let digit_value = c_char::try_from((value_as_i32 >> 6 * i) & 63).unwrap();

        L64A_BUFFER[i] = match digit_value {
            0..=11 => {
                // ./0123456789 for values 0 to 11. b'.' == 46
                46 + digit_value
            }
            12..=37 => {
                // A-Z for values 12 to 37. b'A' == 65, 65-12 == 53
                53 + digit_value
            }
            38..=63 => {
                // a-z for values 38 to 63. b'a' == 97, 97-38 == 59
                59 + digit_value
            }
            _ => unreachable!(), // Guaranteed by taking "& 63" above
        };
    }

    L64A_BUFFER.as_mut_ptr()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/labs.html>.
#[no_mangle]
pub extern "C" fn labs(i: c_long) -> c_long {
    i.abs()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/drand48.html>.
///
/// # Safety
/// The caller must ensure that `param` is convertible to a
/// `&mut [c_ushort; 7]`.
///
/// # Panics
/// Panics if the function is unable to obtain a lock on the generator's global
/// state.
#[no_mangle]
pub unsafe extern "C" fn lcong48(param: *mut c_ushort) {
    let mut xsubi = rand48::xsubi_lock();
    let mut params = rand48::params_mut();

    let param_slice = slice::from_raw_parts(param, 7);

    let xsubi_ref: &[c_ushort; 3] = param_slice[0..3].try_into().unwrap();
    let a_ref: &[c_ushort; 3] = param_slice[3..6].try_into().unwrap();
    let c = param_slice[6];

    *xsubi = xsubi_ref.into();
    params.set(a_ref, c);
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ldiv.html>.
#[repr(C)]
pub struct ldiv_t {
    quot: c_long,
    rem: c_long,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ldiv.html>.
#[no_mangle]
pub extern "C" fn ldiv(numer: c_long, denom: c_long) -> ldiv_t {
    ldiv_t {
        quot: numer / denom,
        rem: numer % denom,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/labs.html>.
#[no_mangle]
pub extern "C" fn llabs(i: c_longlong) -> c_longlong {
    i.abs()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ldiv.html>.
#[repr(C)]
pub struct lldiv_t {
    quot: c_longlong,
    rem: c_longlong,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ldiv.html>.
#[no_mangle]
pub extern "C" fn lldiv(numer: c_longlong, denom: c_longlong) -> lldiv_t {
    lldiv_t {
        quot: numer / denom,
        rem: numer % denom,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/drand48.html>.
///
/// # Panics
/// Panics if the function is unable to obtain a lock on the generator's global
/// state.
#[no_mangle]
pub extern "C" fn lrand48() -> c_long {
    let params = rand48::params();
    let mut xsubi = rand48::xsubi_lock();
    *xsubi = params.step(*xsubi);
    xsubi.get_u31()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/malloc.html>.
#[no_mangle]
pub unsafe extern "C" fn malloc(size: size_t) -> *mut c_void {
    let ptr = platform::alloc(size);
    if ptr.is_null() {
        platform::ERRNO.set(ENOMEM);
    }
    ptr
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/posix_memalign.3.html>.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn memalign(alignment: size_t, size: size_t) -> *mut c_void {
    if alignment.is_power_of_two() {
        let ptr = platform::alloc_align(size, alignment);
        if ptr.is_null() {
            platform::ERRNO.set(ENOMEM);
        }
        ptr
    } else {
        platform::ERRNO.set(EINVAL);
        ptr::null_mut()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mblen.html>.
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mbstowcs.html>.
#[no_mangle]
pub unsafe extern "C" fn mbstowcs(pwcs: *mut wchar_t, mut s: *const c_char, n: size_t) -> size_t {
    let mut state: mbstate_t = mbstate_t {};
    mbsrtowcs(pwcs, &mut s, n, &mut state)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mbtowc.html>.
#[no_mangle]
pub unsafe extern "C" fn mbtowc(pwc: *mut wchar_t, s: *const c_char, n: size_t) -> c_int {
    let mut state: mbstate_t = mbstate_t {};
    mbrtowc(pwc, s, n, &mut state) as c_int
}

fn inner_mktemp<T, F>(name: *mut c_char, suffix_len: c_int, mut attempt: F) -> Option<T>
where
    F: FnMut() -> Option<T>,
{
    let len = unsafe { strlen(name) as c_int };

    if len < 6 || suffix_len > len - 6 {
        platform::ERRNO.set(errno::EINVAL);
        return None;
    }

    for i in (len - suffix_len - 6)..(len - suffix_len) {
        if unsafe { *name.offset(i as isize) } != b'X' as c_char {
            platform::ERRNO.set(errno::EINVAL);
            return None;
        }
    }

    let mut rng = JitterRng::new_with_timer(get_nstime);
    let _ = rng.test_timer();

    for _ in 0..100 {
        let char_iter = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(6)
            .enumerate();
        unsafe {
            for (i, c) in char_iter {
                *name.offset((len as isize) - (suffix_len as isize) - (i as isize) - 1) =
                    c as c_char
            }
        }

        if let result @ Some(_) = attempt() {
            return result;
        }
    }

    platform::ERRNO.set(errno::EEXIST);

    None
}

fn get_nstime() -> u64 {
    unsafe {
        let mut ts = mem::MaybeUninit::uninit();
        Sys::clock_gettime(CLOCK_MONOTONIC, Out::from_uninit_mut(&mut ts));
        ts.assume_init().tv_nsec as u64
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mkdtemp.html>.
#[no_mangle]
pub unsafe extern "C" fn mkdtemp(name: *mut c_char) -> *mut c_char {
    inner_mktemp(name, 0, || {
        let name_c = CStr::from_ptr(name);
        match Sys::mkdir(name_c, 0o700) {
            Ok(()) => Some(name),
            Err(_) => None,
        }
    })
    .unwrap_or(ptr::null_mut())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mkdtemp.html>.
#[no_mangle]
pub unsafe extern "C" fn mkostemp(name: *mut c_char, flags: c_int) -> c_int {
    mkostemps(name, 0, flags)
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/mkstemp.3.html>.
#[no_mangle]
pub unsafe extern "C" fn mkostemps(
    name: *mut c_char,
    suffix_len: c_int,
    mut flags: c_int,
) -> c_int {
    // TODO: Rustify impl

    flags &= !O_ACCMODE;
    flags |= O_RDWR | O_CREAT | O_EXCL;

    inner_mktemp(name, suffix_len, || {
        let name = CStr::from_ptr(name);
        let fd = Sys::open(name, flags, 0o600).or_minus_one_errno();

        if fd >= 0 {
            Some(fd)
        } else {
            None
        }
    })
    .unwrap_or(-1)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mkdtemp.html>.
#[no_mangle]
pub unsafe extern "C" fn mkstemp(name: *mut c_char) -> c_int {
    mkostemps(name, 0, 0)
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/mkstemp.3.html>.
#[no_mangle]
pub unsafe extern "C" fn mkstemps(name: *mut c_char, suffix_len: c_int) -> c_int {
    mkostemps(name, suffix_len, 0)
}

/// See <https://pubs.opengroup.org/onlinepubs/009695399/functions/mktemp.html>.
///
/// # Deprecation
/// The `mktemp()` function was marked as legacy in the Open Group Base
/// Specifications Issue 6, and the function was removed in Issue 7.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn mktemp(name: *mut c_char) -> *mut c_char {
    if inner_mktemp(name, 0, || {
        let name = CStr::from_ptr(name);
        if Sys::access(name, 0) == Err(Errno(ENOENT)) {
            Some(())
        } else {
            None
        }
    })
    .is_none()
    {
        *name = 0;
    }
    name
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/drand48.html>.
///
/// # Panics
/// Panics if the function is unable to obtain a lock on the generator's global
/// state.
#[no_mangle]
pub extern "C" fn mrand48() -> c_long {
    let params = rand48::params();
    let mut xsubi = rand48::xsubi_lock();
    *xsubi = params.step(*xsubi);
    xsubi.get_i32()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/drand48.html>.
///
/// # Safety
/// The caller must ensure that `xsubi` is convertible to a
/// `&mut [c_ushort; 3]`.
///
/// # Panics
/// Panics if the function is unable to obtain a lock on the generator's global
/// state.
#[no_mangle]
pub unsafe extern "C" fn nrand48(xsubi: *mut c_ushort) -> c_long {
    let params = rand48::params();
    let xsubi_mut: &mut [c_ushort; 3] = slice::from_raw_parts_mut(xsubi, 3).try_into().unwrap();
    let new_xsubi_value = params.step(xsubi_mut.into());
    *xsubi_mut = new_xsubi_value.into();
    new_xsubi_value.get_u31()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_memalign.html>.
#[no_mangle]
pub unsafe extern "C" fn posix_memalign(
    memptr: *mut *mut c_void,
    alignment: size_t,
    size: size_t,
) -> c_int {
    const VOID_PTR_SIZE: usize = mem::size_of::<*mut c_void>();

    if alignment % VOID_PTR_SIZE == 0 && alignment.is_power_of_two() {
        let ptr = platform::alloc_align(size, alignment);
        *memptr = ptr;
        if ptr.is_null() {
            ENOMEM
        } else {
            0
        }
    } else {
        *memptr = ptr::null_mut();
        EINVAL
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_openpt.html>.
#[no_mangle]
pub unsafe extern "C" fn posix_openpt(flags: c_int) -> c_int {
    #[cfg(target_os = "redox")]
    let r = open((b"/scheme/pty\0" as *const u8).cast(), O_CREAT);

    #[cfg(target_os = "linux")]
    let r = open((b"/dev/ptmx\0" as *const u8).cast(), flags);

    if r < 0 && platform::ERRNO.get() == ENOSPC {
        platform::ERRNO.set(EAGAIN);
    }

    return r;
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ptsname.html>.
#[no_mangle]
pub unsafe extern "C" fn ptsname(fd: c_int) -> *mut c_char {
    static mut PTS_BUFFER: [c_char; 9 + mem::size_of::<c_int>() * 3 + 1] =
        [0; 9 + mem::size_of::<c_int>() * 3 + 1];
    if ptsname_r(fd, PTS_BUFFER.as_mut_ptr(), PTS_BUFFER.len()) != 0 {
        ptr::null_mut()
    } else {
        PTS_BUFFER.as_mut_ptr()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ptsname.html>.
#[no_mangle]
pub unsafe extern "C" fn ptsname_r(fd: c_int, buf: *mut c_char, buflen: size_t) -> c_int {
    if buf.is_null() {
        platform::ERRNO.set(EINVAL);
        EINVAL
    } else {
        __ptsname_r(fd, buf, buflen)
    }
}

#[cfg(target_os = "redox")]
#[inline(always)]
unsafe fn __ptsname_r(fd: c_int, buf: *mut c_char, buflen: size_t) -> c_int {
    let tty_ptr = unistd::ttyname(fd);

    if !tty_ptr.is_null() {
        if let Ok(name) = CStr::from_ptr(tty_ptr).to_str() {
            let len = name.len();
            if len > buflen {
                platform::ERRNO.set(ERANGE);
                return ERANGE;
            } else {
                // we have checked the string will fit in the buffer
                // so can use strcpy safely
                let s = name.as_ptr().cast();
                ptr::copy_nonoverlapping(s, buf, len);
                return 0;
            }
        }
    }
    platform::ERRNO.get()
}

#[cfg(target_os = "linux")]
#[inline(always)]
unsafe fn __ptsname_r(fd: c_int, buf: *mut c_char, buflen: size_t) -> c_int {
    let mut pty = 0;
    let err = platform::ERRNO.get();

    if ioctl(fd, TIOCGPTN, &mut pty as *mut _ as *mut c_void) == 0 {
        let name = format!("/dev/pts/{}", pty);
        let len = name.len();
        if len > buflen {
            platform::ERRNO.set(ERANGE);
            ERANGE
        } else {
            // we have checked the string will fit in the buffer
            // so can use strcpy safely
            let s = name.as_ptr().cast();
            ptr::copy_nonoverlapping(s, buf, len);
            platform::ERRNO.set(err);
            0
        }
    } else {
        platform::ERRNO.get()
    }
}

unsafe fn put_new_env(insert: *mut c_char) {
    // XXX: Another problem is that `environ` can be set to any pointer, which means there is a
    // chance of a memory leak. But we can check if it was the same as before, like musl does.
    if platform::environ == platform::OUR_ENVIRON.as_mut_ptr() {
        *platform::OUR_ENVIRON.last_mut().unwrap() = insert;
        platform::OUR_ENVIRON.push(core::ptr::null_mut());
        // Likely a no-op but is needed due to Stacked Borrows.
        platform::environ = platform::OUR_ENVIRON.as_mut_ptr();
    } else {
        platform::OUR_ENVIRON.clear();
        platform::OUR_ENVIRON.extend(platform::environ_iter());
        platform::OUR_ENVIRON.push(insert);
        platform::OUR_ENVIRON.push(core::ptr::null_mut());
        platform::environ = platform::OUR_ENVIRON.as_mut_ptr();
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/putenv.html>.
#[no_mangle]
pub unsafe extern "C" fn putenv(insert: *mut c_char) -> c_int {
    assert_ne!(insert, ptr::null_mut(), "putenv(NULL)");
    if let Some((i, _)) = find_env(insert) {
        // XXX: The POSIX manual states that environment variables can be *set* via the `environ`
        // global variable. While we can check if a pointer belongs to our allocator, or check
        // `environ` against a vector which we control, it is likely not worth the effort.
        platform::environ.add(i).write(insert);
    } else {
        put_new_env(insert);
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/qsort.html>.
#[no_mangle]
pub unsafe extern "C" fn qsort(
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/qsort.html>.
// #[no_mangle]
pub unsafe extern "C" fn qsort_r(
    base: *mut c_void,
    nel: size_t,
    width: size_t,
    compar: Option<extern "C" fn(*const c_void, *const c_void, *mut c_void) -> c_int>,
    arg: *mut c_void,
) {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/quick_exit.html>.
#[no_mangle]
pub unsafe extern "C" fn quick_exit(status: c_int) -> ! {
    for i in (0..AT_QUICK_EXIT_FUNCS.len()).rev() {
        if let Some(func) = AT_QUICK_EXIT_FUNCS[i] {
            (func)();
        }
    }

    Sys::exit(status);
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/rand.html>.
#[no_mangle]
pub unsafe extern "C" fn rand() -> c_int {
    match RNG {
        Some(ref mut rng) => rng_sampler().sample(rng),
        None => {
            let mut rng = XorShiftRng::from_seed([1; 16]);
            let ret = rng_sampler().sample(&mut rng);
            RNG = Some(rng);
            ret
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/rand.html>.
///
/// # Deprecation
/// The `rand_r()` function was marked as obsolescent in the Open Group Base
/// Specifications Issue 7, and the function was removed in Issue 8.
#[no_mangle]
pub unsafe extern "C" fn rand_r(seed: *mut c_uint) -> c_int {
    if seed.is_null() {
        errno::EINVAL
    } else {
        // set the type explicitly so this will fail if the array size for XorShiftRng changes
        let seed_arr: [u8; 16] = mem::transmute([*seed; 16 / mem::size_of::<c_uint>()]);

        let mut rng = XorShiftRng::from_seed(seed_arr);
        let ret = rng_sampler().sample(&mut rng);

        *seed = ret as _;

        ret
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/initstate.html>.
// Ported from musl
#[no_mangle]
pub unsafe extern "C" fn random() -> c_long {
    let mut random_state = random::state_lock();

    let k: u32;

    random_state.ensure_x_ptr_init();

    if random_state.n == 0 {
        let x_old = u32::from_ne_bytes(*random_state.x_ptr);
        let x_new = random::lcg31_step(x_old);
        *random_state.x_ptr = x_new.to_ne_bytes();
        k = x_new;
    } else {
        // The non-u32-aligned way of saying x[i] += x[j]...
        let x_i_old = u32::from_ne_bytes(*random_state.x_ptr.add(usize::from(random_state.i)));
        let x_j = u32::from_ne_bytes(*random_state.x_ptr.add(usize::from(random_state.j)));
        let x_i_new = x_i_old.wrapping_add(x_j);
        *random_state.x_ptr.add(usize::from(random_state.i)) = x_i_new.to_ne_bytes();

        k = x_i_new >> 1;

        random_state.i += 1;
        if random_state.i == random_state.n {
            random_state.i = 0;
        }

        random_state.j += 1;
        if random_state.j == random_state.n {
            random_state.j = 0;
        }
    }

    /* Both branches of this function result in a "u31", which will
     * always fit in a c_long. */
    c_long::try_from(k).unwrap()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/realloc.html>.
#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    let new_ptr = platform::realloc(ptr, size);
    if new_ptr.is_null() {
        platform::ERRNO.set(ENOMEM);
    }
    new_ptr
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/realloc.html>.
#[no_mangle]
pub unsafe extern "C" fn reallocarray(ptr: *mut c_void, m: size_t, n: size_t) -> *mut c_void {
    //Handle possible integer overflow in size calculation
    match m.checked_mul(n) {
        Some(size) => realloc(ptr, size),
        None => {
            // For overflowing multiplication, we have to set errno here
            platform::ERRNO.set(ENOMEM);
            ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/realpath.html>.
#[no_mangle]
pub unsafe extern "C" fn realpath(pathname: *const c_char, resolved: *mut c_char) -> *mut c_char {
    let ptr = if resolved.is_null() {
        malloc(limits::PATH_MAX) as *mut c_char
    } else {
        resolved
    };

    let out = slice::from_raw_parts_mut(ptr as *mut u8, limits::PATH_MAX);
    {
        let file = match File::open(CStr::from_ptr(pathname), O_PATH | O_CLOEXEC) {
            Ok(file) => file,
            Err(_) => return ptr::null_mut(),
        };

        let len = out.len();
        // TODO: better error handling
        let read = Sys::fpath(*file, &mut out[..len - 1])
            .map(|read| read as ssize_t)
            .or_minus_one_errno();
        if read < 0 {
            return ptr::null_mut();
        }
        out[read as usize] = 0;
    }

    ptr
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getenv.html>.
// #[no_mangle]
pub unsafe extern "C" fn secure_getenv(name: *const c_char) -> *mut c_char {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/drand48.html>.
///
/// # Safety
/// The caller must ensure that `seed16v` is convertible to a `&[c_ushort; 3]`.
/// Additionally, the caller must ensure that the function has exclusive access
/// to the static buffer it returns; this includes avoiding simultaneous calls
/// to this function.
///
/// # Panics
/// Panics if the function is unable to obtain a lock on the generator's global
/// state.
#[no_mangle]
pub unsafe extern "C" fn seed48(seed16v: *mut c_ushort) -> *mut c_ushort {
    static mut BUFFER: [c_ushort; 3] = [0; 3];

    let mut params = rand48::params_mut();
    let mut xsubi = rand48::xsubi_lock();

    let seed16v_ref: &[c_ushort; 3] = slice::from_raw_parts(seed16v, 3).try_into().unwrap();

    BUFFER = (*xsubi).into();
    *xsubi = seed16v_ref.into();
    params.reset();
    BUFFER.as_mut_ptr()
}

unsafe fn copy_kv(
    existing: *mut c_char,
    key: *const c_char,
    value: *const c_char,
    key_len: usize,
    value_len: usize,
) {
    core::ptr::copy_nonoverlapping(key, existing, key_len);
    core::ptr::write(existing.add(key_len), b'=' as c_char);
    core::ptr::copy_nonoverlapping(value, existing.add(key_len + 1), value_len);
    core::ptr::write(existing.add(key_len + 1 + value_len), 0);
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setenv.html>.
#[no_mangle]
pub unsafe extern "C" fn setenv(
    key: *const c_char,
    value: *const c_char,
    overwrite: c_int,
) -> c_int {
    let key_len = strlen(key);
    let value_len = strlen(value);

    if let Some((i, existing)) = find_env(key) {
        if overwrite == 0 {
            return 0;
        }

        let existing_len = strlen(existing);

        if existing_len >= value_len {
            // Reuse existing element's allocation
            core::ptr::copy_nonoverlapping(value, existing, value_len);
            //TODO: fill to end with zeroes
            core::ptr::write(existing.add(value_len), 0);
        } else {
            // Reuse platform::environ slot, but allocate a new pointer.
            let ptr = platform::alloc(key_len as usize + 1 + value_len as usize + 1) as *mut c_char;
            copy_kv(ptr, key, value, key_len, value_len);
            platform::environ.add(i).write(ptr);
        }
    } else {
        // Expand platform::environ and allocate a new pointer.
        let ptr = platform::alloc(key_len as usize + 1 + value_len as usize + 1) as *mut c_char;
        copy_kv(ptr, key, value, key_len, value_len);
        put_new_env(ptr);
    }

    //platform::free(platform::inner_environ[index] as *mut c_void);

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setkey.html>.
///
/// # Deprecation
/// The `setkey()` function was marked as obsolescent in the Open Group Base
/// Specifications Issue 8.
#[deprecated]
// #[no_mangle]
pub unsafe extern "C" fn setkey(key: *const c_char) {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/initstate.html>.
// Ported from musl. The state parameter is no longer const in newer versions of POSIX.
#[no_mangle]
pub unsafe extern "C" fn setstate(state: *mut c_char) -> *mut c_char {
    let mut random_state = random::state_lock();

    let old_state = random_state.save();
    random_state.load(state.cast::<_>());

    old_state.cast::<_>()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/rand.html>.
#[no_mangle]
pub unsafe extern "C" fn srand(seed: c_uint) {
    RNG = Some(XorShiftRng::from_seed([seed as u8; 16]));
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/drand48.html>.
///
/// # Panics
/// Panics if the function is unable to obtain a lock on the generator's global
/// state.
#[no_mangle]
pub extern "C" fn srand48(seedval: c_long) {
    let mut params = rand48::params_mut();
    let mut xsubi = rand48::xsubi_lock();

    params.reset();
    /* Set the high 32 bits of the 48-bit X_i value to the lower 32 bits
     * of the input argument, and the lower 16 bits to 0x330e, as
     * specified in POSIX. */
    *xsubi = ((u64::from(seedval as u32) << 16) | 0x330e)
        .try_into()
        .unwrap();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/initstate.html>.
// Ported from musl
#[no_mangle]
pub unsafe extern "C" fn srandom(seed: c_uint) {
    let mut random_state = random::state_lock();

    random_state.seed(seed);
}

pub fn is_positive(ch: c_char) -> Option<(bool, isize)> {
    match ch {
        0 => None,
        ch if ch == b'+' as c_char => Some((true, 1)),
        ch if ch == b'-' as c_char => Some((false, 1)),
        _ => Some((true, 0)),
    }
}

pub unsafe fn detect_base(s: *const c_char) -> Option<(c_int, isize)> {
    let first = *s as u8;
    match first {
        0 => None,
        b'0' => {
            let second = *s.offset(1) as u8;
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
        convert_integer(s, 16).map(|(val, idx, overflow)| (val, idx, overflow))
    }
}

pub unsafe fn convert_integer(s: *const c_char, base: c_int) -> Option<(c_ulong, isize, bool)> {
    // -1 means the character is invalid
    #[rustfmt::skip]
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
        // `-1 as usize` is usize::MAX
        // `-1 as u8 as usize` is u8::MAX
        // It extends by the sign bit unless we cast it to unsigned first.
        let val = LOOKUP_TABLE[*s.offset(idx) as u8 as usize];
        if val == -1 || val as c_int >= base {
            break;
        } else {
            if let Some(res) = num
                .checked_mul(base as c_ulong)
                .and_then(|num| num.checked_add(val as c_ulong))
            {
                num = res;
            } else {
                platform::ERRNO.set(ERANGE);
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtod.html>.
#[no_mangle]
pub unsafe extern "C" fn strtod(s: *const c_char, endptr: *mut *mut c_char) -> c_double {
    strto_float_impl!(c_double, s, endptr)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtod.html>.
#[no_mangle]
pub unsafe extern "C" fn strtof(s: *const c_char, endptr: *mut *mut c_char) -> c_float {
    strto_float_impl!(c_float, s, endptr)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtol.html>.
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

// TODO: strtold(), when long double is available

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtol.html>.
#[no_mangle]
pub unsafe extern "C" fn strtoll(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> c_longlong {
    strto_impl!(
        c_longlong,
        true,
        c_longlong::max_value(),
        c_longlong::min_value(),
        s,
        endptr,
        base
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtoul.html>.
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/strtoul.html>.
#[no_mangle]
pub unsafe extern "C" fn strtoull(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> c_ulonglong {
    strto_impl!(
        c_ulonglong,
        false,
        c_ulonglong::max_value(),
        c_ulonglong::min_value(),
        s,
        endptr,
        base
    )
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/system.html>.
#[no_mangle]
pub unsafe extern "C" fn system(command: *const c_char) -> c_int {
    // TODO: rusty error handling?
    //TODO: share code with popen

    // handle shell detection on command == NULL
    if command.is_null() {
        let status = system("exit 0\0".as_ptr() as *const c_char);
        if status == 0 {
            return 1;
        } else {
            return 0;
        }
    }

    let child_pid = unistd::fork();
    if child_pid == 0 {
        let command_nonnull = command as *const u8;

        let shell = "/bin/sh\0".as_ptr();

        let args = [
            "sh\0".as_ptr(),
            "-c\0".as_ptr(),
            command_nonnull,
            ptr::null(),
        ];

        unistd::execv(shell as *const c_char, args.as_ptr() as *const *mut c_char);

        exit(127);

        unreachable!();
    } else if child_pid > 0 {
        let mut wstatus = 0;
        if Sys::waitpid(child_pid, Some(Out::from_mut(&mut wstatus)), 0).or_minus_one_errno() == -1
        {
            return -1;
        }

        wstatus
    } else {
        -1
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/ttyslot.html>.
///
/// # Deprecation
/// The `ttyslot()` function was marked as obsolescent in the Open Group Base
/// Specifications Issue 5, and the function was removed in Issue 6.
#[deprecated]
// #[no_mangle]
pub extern "C" fn ttyslot() -> c_int {
    unimplemented!();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/unlockpt.html>.
#[no_mangle]
pub unsafe extern "C" fn unlockpt(fildes: c_int) -> c_int {
    let mut u: c_int = 0;
    ioctl(fildes, TIOCSPTLCK, &mut u as *mut i32 as *mut c_void)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/unsetenv.html>.
#[no_mangle]
pub unsafe extern "C" fn unsetenv(key: *const c_char) -> c_int {
    if let Some((i, _)) = find_env(key) {
        if platform::environ == platform::OUR_ENVIRON.as_mut_ptr() {
            // No need to worry about updating the pointer, this does not
            // reallocate in any way. And the final null is already shifted back.
            platform::OUR_ENVIRON.remove(i);

            // My UB paranoia.
            platform::environ = platform::OUR_ENVIRON.as_mut_ptr();
        } else {
            platform::OUR_ENVIRON.clear();
            platform::OUR_ENVIRON.extend(
                platform::environ_iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(_, v)| v),
            );
            platform::OUR_ENVIRON.push(core::ptr::null_mut());
            platform::environ = platform::OUR_ENVIRON.as_mut_ptr();
        }
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xsh/valloc.html>.
///
/// # Deprecation
/// The `valloc()` function was marked as obsolescent in the Open Group Base
/// Specifications Issue 5, and the function was removed in Issue 6.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn valloc(size: size_t) -> *mut c_void {
    /* sysconf(_SC_PAGESIZE) is a c_long and may in principle not
     * convert correctly to a size_t. */
    match size_t::try_from(sysconf(_SC_PAGESIZE)) {
        Ok(page_size) => {
            /* valloc() is not supposed to be able to set errno to
             * EINVAL, hence no call to memalign(). */
            let ptr = platform::alloc_align(size, page_size);
            if ptr.is_null() {
                platform::ERRNO.set(ENOMEM);
            }
            ptr
        }
        Err(_) => {
            // A corner case. No errno setting.
            ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wcstombs.html>.
#[no_mangle]
pub unsafe extern "C" fn wcstombs(s: *mut c_char, mut pwcs: *const wchar_t, n: size_t) -> size_t {
    let mut state: mbstate_t = mbstate_t {};
    wcsrtombs(s, &mut pwcs, n, &mut state)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/wctomb.html>.
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
