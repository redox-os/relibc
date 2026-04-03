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

// TODO: Downgrade aux vectors to getauxval constants on Redox, and use a regular struct for
// passing important runtime info between exec (or posix_spawn in the future) calls.

#[cfg(target_os = "redox")]
// XXX: The name AT_CWD is already used in openat... for a completely different purpose.
pub const AT_REDOX_INITIAL_CWD_PTR: usize = 32;
#[cfg(target_os = "redox")]
pub const AT_REDOX_INITIAL_CWD_LEN: usize = 33;

#[cfg(target_os = "redox")]
pub const AT_REDOX_INHERITED_SIGIGNMASK: usize = 34;
#[cfg(all(target_os = "redox", target_pointer_width = "32"))]
pub const AT_REDOX_INHERITED_SIGIGNMASK_HI: usize = 35;
#[cfg(target_os = "redox")]
pub const AT_REDOX_INHERITED_SIGPROCMASK: usize = 36;
#[cfg(all(target_os = "redox", target_pointer_width = "32"))]
pub const AT_REDOX_INHERITED_SIGPROCMASK_HI: usize = 37;

#[cfg(target_os = "redox")]
pub const AT_REDOX_UMASK: usize = 40;

#[cfg(target_os = "redox")]
pub const AT_REDOX_PROC_FD: usize = 41;

#[cfg(target_os = "redox")]
pub const AT_REDOX_THR_FD: usize = 42;

#[cfg(target_os = "redox")]
pub const AT_REDOX_NS_FD: usize = 43;

#[cfg(target_os = "redox")]
pub const AT_REDOX_CWD_FD: usize = 44;
