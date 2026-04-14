//! `cpio.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/cpio.h.html>.

use crate::platform::types::c_int;

/// Read by owner.
pub const C_IRUSR: c_int = 0o0000400;
/// Write by owner.
pub const C_IWUSR: c_int = 0o0000200;
/// Execute by owner.
pub const C_IXUSR: c_int = 0o0000100;
/// Read by group.
pub const C_IRGRP: c_int = 0o0000040;
/// Write by group.
pub const C_IWGRP: c_int = 0o0000020;
/// Execute by group.
pub const C_IXGRP: c_int = 0o0000010;
/// Read by others.
pub const C_IROTH: c_int = 0o0000004;
/// Write by others.
pub const C_IWOTH: c_int = 0o0000002;
/// Execute by others.
pub const C_IXOTH: c_int = 0o0000001;

/// Set user ID.
pub const C_ISUID: c_int = 0o0004000;
/// Set group ID.
pub const C_ISGID: c_int = 0o0002000;
/// On directories, restricted deletion flag.
pub const C_ISVTX: c_int = 0o0001000;

/// Directory.
pub const C_ISDIR: c_int = 0o0040000;
/// FIFO.
pub const C_ISFIFO: c_int = 0o0010000;
/// Regular file.
pub const C_ISREG: c_int = 0o0100000;
/// Block special.
pub const C_ISBLK: c_int = 0o0060000;
/// Character special.
pub const C_ISCHR: c_int = 0o0020000;
/// Reserved.
pub const C_ISCTG: c_int = 0o0110000;
/// Symbolic link.
pub const C_ISLNK: c_int = 0o0120000;
/// Socket.
pub const C_ISSOCK: c_int = 0o0140000;
