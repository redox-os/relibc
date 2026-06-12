use crate::platform::types::c_char;

/// An abstraction over a byte literal to provide a method to convert safely
/// to a `c_char`.
///
/// The abstraction is required so we can contain architecture specific code
/// in a central location.
pub struct ByteLiteral;

impl ByteLiteral {
    /// Casts a byte literal (`u8`) to a `c_char` without using `as`.
    ///
    /// # Panics
    /// If `input` is not within the following range of ascii characters:
    /// - Octal: `040`..=`176`
    /// - Decimal: `30`..=`126`
    /// - Hexadecimal: `20`..=`7E`
    /// - Byte literals: b` `..=b`~`
    pub fn cast_cchar(input: u8) -> c_char {
        match input {
            b' '..=b'~' => {
                // `c_char` is an `i8` on these arches
                #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                {
                    input.cast_signed()
                }
                // `c_char` is already a `u8` on these arches
                #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
                {
                    input.into()
                }
            }
            _ => panic!("Not a printable ascii character!"),
        }
    }
}
