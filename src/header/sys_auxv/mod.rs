//! sys/auxv.h implementation

use crate::platform::types::*;

pub const AT_NULL: usize = 0; /* End of vector */
pub const AT_IGNORE: usize = 1; /* Entry should be ignored */
pub const AT_EXECFD: usize = 2; /* File descriptor of program */
pub const AT_PHDR: usize = 3; /* Program headers for program */
pub const AT_PHENT: usize = 4; /* Size of program header entry */
pub const AT_PHNUM: usize = 5; /* Number of program headers */
pub const AT_PAGESZ: usize = 6; /* System page size */
pub const AT_BASE: usize = 7; /* Base address of interpreter */
pub const AT_FLAGS: usize = 8; /* Flags */
pub const AT_ENTRY: usize = 9; /* Entry point of program */
pub const AT_NOTELF: usize = 10; /* Program is not ELF */
pub const AT_UID: usize = 11; /* Real uid */
pub const AT_EUID: usize = 12; /* Effective uid */
pub const AT_GID: usize = 13; /* Real gid */
pub const AT_EGID: usize = 14; /* Effective gid */
pub const AT_CLKTCK: usize = 17; /* Frequency of times() */
pub const AT_PLATFORM: usize = 15; /* String identifying platform.  */
pub const AT_HWCAP: usize = 16; /* Machine-dependent hints about */
pub const AT_FPUCW: usize = 18; /* Used FPU control word.  */
pub const AT_DCACHEBSIZE: usize = 19; /* Data cache block size.  */
pub const AT_ICACHEBSIZE: usize = 20; /* Instruction cache block size.  */
pub const AT_UCACHEBSIZE: usize = 21; /* Unified cache block size.  */
pub const AT_IGNOREPPC: usize = 22; /* Entry should be ignored.  */
pub const AT_BASE_PLATFORM: usize = 24; /* String identifying real platforms.*/
pub const AT_RANDOM: usize = 25; /* Address of 16 random bytes.  */
pub const AT_HWCAP2: usize = 26; /* More machine-dependent hints about*/
pub const AT_EXECFN: usize = 31; /* Filename of executable.  */

#[no_mangle]
pub extern "C" fn getauxval(_t: c_ulong) -> c_ulong {
    0
}
