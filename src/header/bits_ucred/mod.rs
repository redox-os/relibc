use crate::platform::types::{gid_t, pid_t, uid_t};

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man7/unix.7.html>.
///
/// Represents UNIX credentials.
#[repr(C)]
#[derive(Clone, Debug)]
// FIXME: CheckVsLibcCrate
pub struct ucred {
    /// Process ID of the sending process.
    pub pid: pid_t,
    /// User ID of the sending process.
    pub uid: uid_t,
    /// Group ID of the sending process.
    pub gid: gid_t,
}
