//! Casting between different types.
//!
//! These abstractions allow us to contain architecture specific code in a
//! central location reducing verbosity around the codebase, easing
//! maintenance and improving readability.
//!
//! They also allow the elimination of `as` casts in cases where the user is
//! certain that the transformation between types is within intended bounds.

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

/// An abstraction for converting a `u8` pointer to a `c_char` pointer.
pub struct U8PtrToCCharPtr;

impl U8PtrToCCharPtr {
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

/// An abstraction for converting a `c_char` pointer to a `u8` pointer.
pub struct CCharPtrToU8Ptr;

impl CCharPtrToU8Ptr {
    /// A method that casts a `*const c_char` to a `*const u8`.
    ///
    /// It is the caller's responsibility to understand and ensure that casting
    /// from `c_char` to `u8` is intentional for the applicable architectures.
    ///
    /// # Panics
    /// Unsupported architectures will panic.
    pub fn cast_const(input: *const c_char) -> *const u8 {
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
    /// A method that casts a `*const c_char` to a `*mut u8`.
    ///
    /// It is the caller's responsibility to understand and ensure that casting
    /// from `c_char` to `u8` is intentional for the applicable architectures.
    ///
    /// # Panics
    /// Unsupported architectures will panic.
    pub fn cast_mut(input: *const c_char) -> *mut u8 {
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

/// An abstraction for converting a `c_char` pointer to a `u8`.
pub struct CCharPtrToU8;

impl CCharPtrToU8 {
    /// A method that casts a `*const c_char` to a `u8`.
    ///
    /// It is the caller's responsibility to understand and ensure that casting
    /// from `c_char` to `u8` is intentional for the applicable architectures.
    ///
    /// # Panics
    /// Unsupported architectures will panic.
    ///
    /// # Safety
    /// `input` must not be null.
    pub unsafe fn from_const(input: *const c_char) -> u8 {
        // `c_char` is an `i8` on these arches
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        {
            return unsafe { *input }.cast_unsigned();
        }
        // `c_char` is already a `u8` on these arches
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        {
            return unsafe { *input };
        }
        panic!("Arch not supported!")
    }
}

/// An abstraction for converting a `c_char` to a `u8`.
pub struct CCharToU8;

impl CCharToU8 {
    /// A method that casts a `c_char` to a `u8`.
    ///
    /// It is the caller's responsibility to understand and ensure that casting
    /// from `c_char` to `u8` is intentional for the applicable architectures.
    ///
    /// # Panics
    /// Unsupported architectures will panic.
    pub fn cast(input: c_char) -> u8 {
        // `c_char` is an `i8` on these arches
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        {
            return input.cast_unsigned();
        }
        // `c_char` is already a `u8` on these arches
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        {
            return input;
        }
        panic!("Arch not supported!")
    }
}

// A trait intended to indicate infallible transformations when we know the
// value is within the expected bounds.
pub trait FromExt<T>: Sized {
    fn inf_from(value: T) -> Self;
}

impl FromExt<i32> for u16 {
    /// Infallible cast of `i32` to `u16`.
    ///
    /// # Panics
    /// If the `i32` value is negative or larger than `u16::MAX`.
    fn inf_from(value: i32) -> Self {
        Self::try_from(value).expect("should be within bounds")
    }
}

impl FromExt<u32> for usize {
    /// Infallible cast of `u32` to `usize`.
    ///
    /// # Panics
    /// If the `u32` value is larger than the platform specific `usize` value.
    fn inf_from(value: u32) -> Self {
        Self::try_from(value).expect("should be within bounds")
    }
}

impl FromExt<u64> for usize {
    /// Infallible cast of `u64` to `usize`.
    ///
    /// # Panics
    /// If the `u64` value is larger than the platform specific `usize` value.
    fn inf_from(value: u64) -> Self {
        Self::try_from(value).expect("should be within bounds")
    }
}

impl FromExt<usize> for u8 {
    /// Infallible cast of `usize` to `u8`.
    ///
    /// # Panics
    /// If the `usize` value is larger than `u8::MAX`.
    fn inf_from(value: usize) -> Self {
        Self::try_from(value).expect("should be within bounds")
    }
}

impl FromExt<usize> for u16 {
    /// Infallible cast of `usize` to `u16`.
    ///
    /// # Panics
    /// If the `usize` value is larger than `u16::MAX`.
    fn inf_from(value: usize) -> Self {
        Self::try_from(value).expect("should be within bounds")
    }
}

impl FromExt<usize> for i32 {
    /// Infallible cast of `usize` to `i32`.
    ///
    /// # Panics
    /// If the `usize` value is larger than `i32::MAX`.
    fn inf_from(value: usize) -> Self {
        Self::try_from(value).expect("should be within bounds")
    }
}
