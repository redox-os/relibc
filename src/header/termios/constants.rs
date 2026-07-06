// These constants have values shared between Redox and Linux.

/* c_cc { */
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/termios.3.html>.
///
/// SWTCH character (not supported under Linux).
/// Used in System V to switch shells in `shell layers`, a predecessor to shell
/// job control.
pub const VSWTCH: usize = 7;
/// SUSP character.
/// Send `SIGTSTP` signal.
/// Canonical and Non-Canonical mode.
pub const VSUSP: usize = 10;
/// Size of the array `c_cc` for control characters.
pub const NCCS: usize = 32;
/* } c_cc */

/* c_iflag { */
/// Ignore break condition.
pub const IGNBRK: usize = 0o000_001;
/// Signal interrupt on break.
pub const BRKINT: usize = 0o000_002;
/// Ignore characters with parity errors.
pub const IGNPAR: usize = 0o000_004;
/// Mark parity errors.
pub const PARMRK: usize = 0o000_010;
/// Enable input parity check.
pub const INPCK: usize = 0o000_020;
/// Strip character.
pub const ISTRIP: usize = 0o000_040;
/// Map NL to CR on input.
pub const INLCR: usize = 0o000_100;
/// Ignore CR.
pub const IGNCR: usize = 0o000_200;
/// Map CR to NL on input.
pub const ICRNL: usize = 0o000_400;
/// Enable any character to restart output.
pub const IXANY: usize = 0o004_000;
/* } c_iflag */
