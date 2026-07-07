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

/* c_oflag { */
/// Post-process output.
pub const OPOST: usize = 0o000_001;
/// Map CR to NL on output.
pub const OCRNL: usize = 0o000_010;
/// No CR output at column 0.
pub const ONOCR: usize = 0o000_020;
/// NL performs CR function.
pub const ONLRET: usize = 0o000_040;
/// Use fill characters for delay.
pub const OFILL: usize = 0o000_100;
/// Fill is DEL.
pub const OFDEL: usize = 0o000_200;
/* } c_oflag */

/* c_cflag { */
/// Hang up.
pub const B0: usize = 0o000_000;
/// 50 baud.
pub const B50: usize = 0o000_001;
/// 75 baud.
pub const B75: usize = 0o000_002;
/// 110 baud.
pub const B110: usize = 0o000_003;
/// 134.5 baud.
pub const B134: usize = 0o000_004;
/// 150 baud.
pub const B150: usize = 0o000_005;
/// 200 baud.
pub const B200: usize = 0o000_006;
/// 300 baud.
pub const B300: usize = 0o000_007;
/// 600 baud.
pub const B600: usize = 0o000_010;
/// 1200 baud.
pub const B1200: usize = 0o000_011;
/// 1800 baud.
pub const B1800: usize = 0o000_012;
/// 2400 baud.
pub const B2400: usize = 0o000_013;
/// 4800 baud.
pub const B4800: usize = 0o000_014;
/// 9600 baud.
pub const B9600: usize = 0o000_015;
/// 19200 baud.
pub const B19200: usize = 0o000_016;
/// 38400 baud.
pub const B38400: usize = 0o000_017;

/// 5 bits.
pub const CS5: usize = 0o000_000;
/* } c_clfag */

/* c_lflag { */
/// Enable echo.
pub const ECHO: usize = 0o000_010;
/* } c_lflag */

// POSIX extensions
/// Sentinel value to disable a control char.
pub const _POSIX_VDISABLE: u8 = 0;
