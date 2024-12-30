//! Helper functions for pseudorandom number generation using LCG, see https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/functions/drand48.html

use crate::{
    platform::types::*,
    sync::{rwlock::{self, RwLock}, Mutex, MutexGuard},
};

/// A 48-bit integer, used for the 48-bit arithmetic in these functions.
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

impl From<&mut [c_ushort; 3]> for U48 {
    fn from(value: &mut [c_ushort; 3]) -> Self {
        Self::from(&*value)
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

/// The a and c parameters of an LCG.
#[derive(Default)]
#[repr(C)]
pub struct Params {
    pub a: U48,
    pub c: u16,
}

impl Params {
    pub const fn new() -> Self {
        // Default values as specified in POSIX
        Params {
            a: U48(0x5deece66d),
            c: 0xb,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// For use in lcong48().
    pub fn set(&mut self, a: &[c_ushort; 3], c: c_ushort) {
        self.a = a.into();
        self.c = c as u16; // Per POSIX, discard higher bits in case unsigned short is larger than u16
    }

    pub fn step(&self, xsubi: U48) -> U48 {
        /* The recurrence relation of the linear congruential generator,
         * X_(n+1) = (a * X_n + c) % m,
         * with m = 2**48. The multiplication and addition can overflow a u64, but
         * we just let it wrap since we take mod 2**48 anyway. */
        (u64::from(self.a)
            .wrapping_mul(u64::from(xsubi))
            .wrapping_add(u64::from(self.c))
            & 0xffff_ffff_ffff)
            .try_into()
            .unwrap()
    }
}

static PARAMS: RwLock<Params> = RwLock::<Params>::new(Params::new());

/// Immediately get the global [`Params`] lock for reading, or panic if unsuccessful.
pub fn params<'a>() -> rwlock::ReadGuard<'a, Params> {
    PARAMS
        .try_read()
        .expect("unable to acquire LCG parameter lock")
}

/// Immediately get the global [`Params`] lock for writing, or panic if unsuccessful.
pub fn params_mut<'a>() -> rwlock::WriteGuard<'a, Params> {
    PARAMS
        .try_write()
        .expect("unable to acquire LCG parameter lock")
}

/// Immediately get the global X_i lock, or panic if unsuccessful.
pub fn xsubi_lock<'a>() -> MutexGuard<'a, U48> {
    static XSUBI: Mutex<U48> = Mutex::<U48>::new(U48(0));

    XSUBI.try_lock().expect("unable to acquire LCG X_i lock")
}
