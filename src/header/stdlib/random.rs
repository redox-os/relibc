//! Helper functions for random() and friends, see https://pubs.opengroup.org/onlinepubs/7908799/xsh/initstate.html
// Ported from musl's implementation (src/prng/random.c)

use crate::{
    platform::types::c_uint,
    sync::{Mutex, MutexGuard},
};
use core::{cell::SyncUnsafeCell, convert::TryFrom, ptr};

// This is the default buffer for X. Not guarded by mutex as a mutable pointer to this is available to the user through initstate() and setstate().
#[rustfmt::skip]
pub static DEFAULT_X: SyncUnsafeCell<[u32; 32]> = SyncUnsafeCell::new([
    0x00000000, 0x5851f42d, 0xc0b18ccf, 0xcbb5f646,
    0xc7033129, 0x30705b04, 0x20fd5db4, 0x9a8b7f78,
    0x502959d8, 0xab894868, 0x6c0356a7, 0x88cdb7ff,
    0xb477d43f, 0x70a3a52b, 0xa8e4baf1, 0xfd8341fc,
    0x8ae16fd9, 0x742d2f7a, 0x0d1f0796, 0x76035e09,
    0x40f7702c, 0x6fa72ca5, 0xaaa84157, 0x58a0df74,
    0xc74a0364, 0xae533cc4, 0x04185faf, 0x6de3b115,
    0x0cab8628, 0xf043bfa4, 0x398150e9, 0x37521657,
]);

pub struct State {
    pub x_ptr: *mut [u8; 4],
    pub n: u8,
    pub i: u8,
    pub j: u8,
}

// Necessary because raw pointers are not Send
unsafe impl Send for State {}

impl State {
    /// To be called in any function that may read from X_PTR
    pub unsafe fn ensure_x_ptr_init(&mut self) {
        if self.x_ptr.is_null() {
            let x_u32_ptr: *mut u32 = unsafe { DEFAULT_X.get().cast::<u32>().add(1) };
            self.x_ptr = x_u32_ptr.cast::<[u8; 4]>();
        }
    }

    pub unsafe fn save(&mut self) -> *mut [u8; 4] {
        unsafe { self.ensure_x_ptr_init() };

        let stash_value: u32 =
            (u32::from(self.n) << 16) | (u32::from(self.i) << 8) | u32::from(self.j);
        unsafe { *self.x_ptr.offset(-1) = stash_value.to_ne_bytes() };
        unsafe { self.x_ptr.offset(-1) }
    }

    pub unsafe fn load(&mut self, state_ptr: *mut [u8; 4]) {
        let stash_value = u32::from_ne_bytes(unsafe { *state_ptr });
        self.x_ptr = unsafe { state_ptr.offset(1) };

        /* This calculation of n does not have a bit mask in the musl
         * original, in principle resulting in a u16, but obtaining a value
         * larger than 63 can probably be dismissed as pathological. */
        self.n = u8::try_from((stash_value >> 16) & 0xff).unwrap();

        // i and j calculations are straight from musl
        self.i = u8::try_from((stash_value >> 8) & 0xff).unwrap();
        self.j = u8::try_from(stash_value & 0xff).unwrap();
    }

    pub unsafe fn seed(&mut self, seed: c_uint) {
        unsafe { self.ensure_x_ptr_init() };

        let mut s = seed as u64;

        if self.n == 0 {
            unsafe { *self.x_ptr = (s as u32).to_ne_bytes() };
        } else {
            self.i = if self.n == 31 || self.n == 7 { 3 } else { 1 };

            self.j = 0;

            for k in 0..usize::from(self.n) {
                s = lcg64_step(s);

                // Conversion will always succeed (value is a 32-bit right-
                // shift of a 64-bit integer).
                unsafe { *self.x_ptr.add(k) = u32::try_from(s >> 32).unwrap().to_ne_bytes() };
            }

            // ensure X contains at least one odd number
            unsafe { *self.x_ptr = (u32::from_ne_bytes(*self.x_ptr) | 1).to_ne_bytes() };
        }
    }
}

pub fn state_lock<'a>() -> MutexGuard<'a, State> {
    static STATE: Mutex<State> = Mutex::new(State {
        /* As such, random() and related functions work on u32 values, but POSIX
         * allows the user to supply a custom state data array as a `char *`
         * with no requirements on alignment. Thus, we must assume the worst in
         * terms of alignment and convert back and forth from [u8; 4].
         *
         * Also, unlike in C, we can't take the address of the initializing
         * array outside of a function. */
        x_ptr: ptr::null_mut(),
        /* N needs to accommodate values up to 63, corresponding to the maximum
         * state array size of 256 bytes. I and J must be able to accommodate
         * values less than or equal to N. */
        n: 31,
        i: 3,
        j: 0,
    });

    STATE.try_lock().expect("unable to acquire PRNG lock")
}

pub fn lcg31_step(x: u32) -> u32 {
    1103515245_u32.wrapping_mul(x).wrapping_add(12345_u32) & 0x7fffffff
}

pub fn lcg64_step(x: u64) -> u64 {
    6364136223846793005_u64.wrapping_mul(x).wrapping_add(1_u64)
}
