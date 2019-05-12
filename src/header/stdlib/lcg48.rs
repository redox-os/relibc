//! Helper functions for pseudorandom number generation using LCG, see http://pubs.opengroup.org/onlinepubs/7908799/xsh/drand48.html

use platform::types::*;

/* The current element of the linear congruential generator's sequence. Any
 * function that sets this variable must ensure that only the lower 48 bits get
 * set. */
pub static mut XI: u64 = 0;

/* Multiplier and addend, which may be set through lcong48(). Default values as
 * specified in POSIX. */
pub static mut A: u64 = 0x5deece66d;
pub static mut C: u16 = 0xb;

/// Advances the linear congruential generator to the next element in its
/// sequence.
pub unsafe fn generator_step() {
    /* The recurrence relation of the linear congruential generator,
     * X_(n+1) = (a * X_n + c) % m,
     * with m = 2**48. The multiplication and addition can overflow a u64, but
     * we just let it wrap since we take mod 2**48 anyway. */
    XI = A.wrapping_mul(XI).wrapping_add(u64::from(C)) & 0xffff_ffff_ffff;
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
