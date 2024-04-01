//! Helper functions for pseudorandom number generation using LCG, see https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/functions/drand48.html

use crate::{platform::types::*, sync::Mutex};

const STATE_DEFAULT_VALUE: State = State {
    xsubi: U48(0),
    a: A_DEFAULT_VALUE,
    c: 0xb,
};
pub static STATE: Mutex<State> = Mutex::<State>::new(STATE_DEFAULT_VALUE);
pub const CONTENTION_MSG: &str = "attempted unsafe multithreaded access";

// TODO: replace static mut?
pub static mut SEED48_BUFFER: [c_ushort; 3] = [0; 3];

/* Multiplier and addend, which may be set through lcong48(). Default
 * values as specified in POSIX. */
const A_DEFAULT_VALUE: U48 = U48(0x5deece66d);
const C_DEFAULT_VALUE: u16 = 0xb;

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct U48(u64);

impl From<&[c_ushort; 3]> for U48 {
    fn from(value: &[c_ushort; 3]) -> Self {
        /* Cast via u16 to ensure we get only the lower 16 bits of each
         * element, as specified by POSIX. */
        Self {
            0: u64::from(value[0] as u16)
                | (u64::from(value[1] as u16) << 16)
                | (u64::from(value[2] as u16) << 32),
        }
    }
}

impl TryFrom<u64> for U48 {
    type Error = u64;

    fn try_from(value: u64) -> Result<Self, u64> {
        if value < 0x1_0000_0000_0000 {
            Ok(Self { 0: value })
        } else {
            Err(value)
        }
    }
}

impl From<U48> for u64 {
    fn from(value: U48) -> Self {
        value.0
    }
}

impl From<U48> for [c_ushort; 3] {
    fn from(value: U48) -> Self {
        [
            // "as u16" in case c_ushort is larger than u16
            (value.0 as u16).into(),
            ((value.0 >> 16) as u16).into(),
            ((value.0 >> 32) as u16).into(),
        ]
    }
}

impl U48 {
    /// Get a C `double` in the interval [0.0, 1.0) (for `drand48()` and `erand48()`).
    pub fn get_f64(self) -> c_double {
        /* We set the exponent to 0, and the 48-bit integer is copied into the high
         * 48 of the 52 significand bits. The value then lies in the range
         * [1.0, 2.0), from which we simply subtract 1.0. */
        f64::from_bits(0x3ff0_0000_0000_0000_u64 | (self.0 << 4)) - 1.0
    }

    /// Get the high 31 bits (for `lrand48()` and `nrand48()`).
    pub fn get_u31(self) -> c_long {
        (self.0 >> 17).try_into().unwrap()
    }

    /// Get the high 32 bits, signed (for `mrand48()` and `jrand48()`).
    pub fn get_i32(self) -> c_long {
        // Cast via i32 to ensure we get the sign correct
        ((self.0 >> 16) as i32).into()
    }
}

#[derive(Default)]
#[repr(C)]
pub struct State {
    pub xsubi: U48,
    pub a: U48,
    pub c: u16,
}

impl State {
    pub fn step(&mut self) -> U48 {
        let new_xsubi_value = step(self.xsubi, self.a, self.c);
        self.xsubi = new_xsubi_value;
        new_xsubi_value
    }

    pub fn step_other(&self, other: &mut [c_ushort; 3]) -> U48 {
        let old_xsubi_value = U48::from(&*other);
        let new_xsubi_value: U48 = step(old_xsubi_value, self.a, self.c);
        *other = new_xsubi_value.into();
        new_xsubi_value
    }

    pub fn reset_a_and_c(&mut self) {
        self.a = A_DEFAULT_VALUE;
        self.c = C_DEFAULT_VALUE;
    }
}

fn step(xsubi: U48, a: U48, c: u16) -> U48 {
    /* The recurrence relation of the linear congruential generator,
     * X_(n+1) = (a * X_n + c) % m,
     * with m = 2**48. The multiplication and addition can overflow a u64, but
     * we just let it wrap since we take mod 2**48 anyway. */
    (u64::from(a)
        .wrapping_mul(u64::from(xsubi))
        .wrapping_add(u64::from(c))
        & 0xffff_ffff_ffff)
        .try_into()
        .unwrap()
}
