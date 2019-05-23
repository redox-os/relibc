//! Helper functions for pseudorandom number generation using LCG, see https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/functions/drand48.html

use platform::types::*;

/* The current element of the linear congruential generator's sequence. Any
 * function that sets this variable must ensure that only the lower 48 bits get
 * set. */
pub static mut XI: u64 = 0;

// Used by seed48() (returns a pointer to this array).
pub static mut STASHED_XI: [c_ushort; 3] = [0; 3];

/* Multiplier and addend, which may be set through lcong48(). Default values as
 * specified in POSIX. */
const A_DEFAULT: u64 = 0x5deece66d;
const C_DEFAULT: u16 = 0xb;

pub static mut A: u64 = A_DEFAULT;
pub static mut C: u16 = C_DEFAULT;

/// Gets the next element in the linear congruential generator's
/// sequence.
pub unsafe fn next_x(x: u64) -> u64 {
    /* The recurrence relation of the linear congruential generator,
     * X_(n+1) = (a * X_n + c) % m,
     * with m = 2**48. The multiplication and addition can overflow a u64, but
     * we just let it wrap since we take mod 2**48 anyway. */
    A.wrapping_mul(x).wrapping_add(u64::from(C)) & 0xffff_ffff_ffff
}

/// Get a C `double` from a 48-bit integer (for `drand48()` and `erand48()`).
pub fn x_to_float64(x: u64) -> c_double {
    /* We set the exponent to 0, and the 48-bit integer is copied into the high
     * 48 of the 52 significand bits. The value then lies in the range
     * [1.0, 2.0), from which we simply subtract 1.0. */
    f64::from_bits(0x3ff0_0000_0000_0000_u64 | (x << 4)) - 1.0f64
}

/// Get the high 31 bits of a 48-bit integer (for `lrand48()` and `nrand48()`).
pub fn x_to_uint31(x: u64) -> c_long {
    (x >> 17) as c_long
}

/// Get the high 32 bits, signed, of a 48-bit integer (for `mrand48()` and
/// `jrand48()`).
pub fn x_to_int32(x: u64) -> c_long {
    // Cast via i32 to ensure we get the sign correct
    (x >> 16) as i32 as c_long
}

/// Build a 48-bit integer from a size-3 array of unsigned short.
/// 
/// Takes a pointer argument due to the inappropriate C function
/// signatures generated from Rust's sized arrays, see
/// https://github.com/eqrion/cbindgen/issues/171
pub unsafe fn ushort_arr3_to_uint48(arr_ptr: *const c_ushort) -> u64 {
    let arr = [*arr_ptr.offset(0), *arr_ptr.offset(1), *arr_ptr.offset(2)];
    
    /* Cast via u16 to ensure we get only the lower 16 bits of each
     * element, as specified by POSIX. */
    u64::from(arr[0] as u16) | (u64::from(arr[1] as u16) << 16) | (u64::from(arr[2] as u16) << 32)
}

/// Set a size-3 array of unsigned short from a 48-bit integer.
pub unsafe fn set_ushort_arr3_from_uint48(arr_ptr: *mut c_ushort, value: u64) {
    *arr_ptr.offset(0) = c_ushort::from(value as u16);
    *arr_ptr.offset(1) = c_ushort::from((value >> 16) as u16);
    *arr_ptr.offset(2) = c_ushort::from((value >> 32) as u16);
}

/// Used by `srand48()` and `seed48()`.
pub unsafe fn reset_a_and_c() {
    A = A_DEFAULT;
    C = C_DEFAULT;
}
