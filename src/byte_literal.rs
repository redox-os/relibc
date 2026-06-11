use crate::platform::types::c_char;

pub struct ByteLiteral(u8);

impl ByteLiteral {
    pub fn cast_unchecked(input: u8) -> c_char {
        match input {
            b' '..=b'~' => {
                #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                {
                    input.cast_signed()
                }
                #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
                {
                    input.cast_unsigned()
                }
            },
            _ => panic!("Not a printable ascii character!"),
        }
    }
}
