//! Helper functions for pseudorandom number generation using LCG, see https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/functions/drand48.html

use crate::platform::types::*;

/* The default element buffer for the linear congruential generator's
 * sequence. Implemented using a c_ushort array for consistency between
 * the drand48()/lrand48()/mrand48() and erand48()/nrand48()/jrand48()
 * functions, and with SEED48_XSUBI (see below). */
pub static mut DEFAULT_XSUBI: [c_ushort; 3] = [0; 3];

// Used by seed48() (returns a pointer to this array).
pub static mut SEED48_XSUBI: [c_ushort; 3] = [0; 3];

/* Multiplier and addend, which may be set through lcong48(). Default
 * values as specified in POSIX. */
const A_DEFAULT_VALUE: u64 = 0x5deece66d;
const C_DEFAULT_VALUE: u16 = 0xb;

pub static mut A: u64 = A_DEFAULT_VALUE;
pub static mut C: u16 = C_DEFAULT_VALUE;

/// Used by `srand48()` and `seed48()`.
pub unsafe fn reset_a_and_c() {
    A = A_DEFAULT_VALUE;
    C = C_DEFAULT_VALUE;
}

/// Build a 48-bit integer from a size-3 array of unsigned short.
///
/// Pointers to c_ushort can be converted to &[c_ushort; 3] by taking
/// &*(YOUR_C_USHORT_POINTER as *const [c_ushort; 3])
///
/// See also this cbindgen issue for why the stdlib functions can't just
/// have an xsubi: *mut [c_ushort; 3] parameter:
/// https://github.com/eqrion/cbindgen/issues/171
pub fn u48_from_ushort_arr3(arr: &[c_ushort; 3]) -> u64 {
    /* Cast via u16 to ensure we get only the lower 16 bits of each
     * element, as specified by POSIX. */
    u64::from(arr[0] as u16) | (u64::from(arr[1] as u16) << 16) | (u64::from(arr[2] as u16) << 32)
}

/// Make a size-3 array of unsigned short from a 48-bit integer.
pub fn ushort_arr3_from_u48(value: u64) -> [c_ushort; 3] {
    [
        c_ushort::from(value as u16),
        c_ushort::from((value >> 16) as u16),
        c_ushort::from((value >> 32) as u16),
    ]
}

/// Advances the buffer from the input argument to the next element in
/// the linear congruential generator's sequence.
///
/// Modifies the passed argument in-place and returns the new value as a
/// u64.
pub unsafe fn generator_step(xsubi: &mut [c_ushort; 3]) -> u64 {
    let old_xsubi_value: u64 = u48_from_ushort_arr3(xsubi);

    /* The recurrence relation of the linear congruential generator,
     * X_(n+1) = (a * X_n + c) % m,
     * with m = 2**48. The multiplication and addition can overflow a
     * u64, but we just let it wrap since we take mod 2**48 anyway. */
    let new_xsubi_value: u64 =
        A.wrapping_mul(old_xsubi_value).wrapping_add(u64::from(C)) & 0xffff_ffff_ffff;

    *xsubi = ushort_arr3_from_u48(new_xsubi_value);
    new_xsubi_value
}

/// Get a C `double` from a 48-bit integer (for `drand48()` and
/// `erand48()`).
pub fn f64_from_x(x: u64) -> c_double {
    /* We set the exponent to 0, and the 48-bit integer is copied into the high
     * 48 of the 52 significand bits. The value then lies in the range
     * [1.0, 2.0), from which we simply subtract 1.0. */
    f64::from_bits(0x3ff0_0000_0000_0000_u64 | (x << 4)) - 1.0
}

/// Get the high 31 bits of a 48-bit integer (for `lrand48()` and
/// `nrand48()`).
pub fn u31_from_x(x: u64) -> c_long {
    (x >> 17) as c_long
}

/// Get the high 32 bits, signed, of a 48-bit integer (for `mrand48()`
/// and `jrand48()`).
pub fn i32_from_x(x: u64) -> c_long {
    // Cast via i32 to ensure we get the sign correct
    c_long::from((x >> 16) as i32)
}
