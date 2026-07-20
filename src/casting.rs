//! Casting between types that differ per architecture.
//!
//! These abstractions allow us to contain architecture specific code in a
//! central location reducing verbosity around the codebase and easing
//! maintenance.

use crate::platform::types::c_char;

/// An abstraction over a byte literal to provide a method to convert safely
/// to a `c_char`.
pub struct ByteLiteral;

impl ByteLiteral {
    /// Casts a byte literal (`u8`) to a `c_char` without using `as`.
    ///
    /// # Panics
    /// If `input` is not within the following range of ascii characters:
    /// - Octal: `040`..=`176`
    /// - Decimal: `30`..=`126`
    /// - Hexadecimal: `20`..=`7E`
    /// - Byte literals: The space character (` `) upto and including tilde (`~`)
    pub const fn cast_cchar(input: u8) -> c_char {
        match input {
            b' '..=b'~' => {
                // `c_char` is an `i8` on these arches
                #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                {
                    // Casting `u8` to `i8` is always safe when within the
                    // printable ascii range
                    input.cast_signed()
                }
                // `c_char` is already a `u8` on these arches
                #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
                {
                    input
                }
            }
            _ => panic!("Not a printable ascii character!"),
        }
    }
}

/// An abstraction for converting a pointer to a `c_char` pointer.
pub struct CCharPtr;

impl CCharPtr {
    /// A method that casts a `*const u8` to a `*const c_char`.
    ///
    /// It is the caller's responsibility to understand and ensure that casting
    /// from `u8` to `i8` is intentional for the applicable architectures.
    ///
    /// # Panics
    /// Unsupported architectures will panic.
    pub fn cast_const(input: *const u8) -> *const c_char {
        // `c_char` is an `i8` on these arches
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        {
            return input.cast();
        }
        // `c_char` is already a `u8` on these arches
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        {
            return input;
        }
        panic!("Arch not supported!")
    }
    /// A method that casts a `*const u8` to a `*mut c_char`.
    ///
    /// It is the caller's responsibility to understand and ensure that casting
    /// from `u8` to `i8` is intentional for the applicable architectures.
    ///
    /// # Panics
    /// Unsupported architectures will panic.
    pub fn cast_mut(input: *const u8) -> *mut c_char {
        // `c_char` is an `i8` on these arches
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        {
            // TODO replace `as` casting with something better
            return input as *mut _;
        }
        // `c_char` is already a `u8` on these arches
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        {
            return input.cast_mut();
        }
        panic!("Arch not supported!")
    }
}
