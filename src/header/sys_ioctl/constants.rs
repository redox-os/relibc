// These constants have the same values on both redox and linux.
// TODO add more from linux as and when redox supports them.

use crate::platform::types::c_ulong;

/// Equivalent to `tcgetattr(fd, argp)`.
/// Get the current serial port settings.
pub const TCGETS: c_ulong = 0x5401;
/// Equivalent to `tcsetattr(fd, TCSANOW, argp)`.
/// Set the current serial port settings.
pub const TCSETS: c_ulong = 0x5402;
/// Equivalent to `tcsetattr(fd, TCSADRAIN, argp)`.
/// Allow the output buffer to drain, and set the current serial port settings.
pub const TCSETSW: c_ulong = 0x5403;
/// Equivalent to `tcsetattr(fd, TCSAFLUSH, argp)`.
/// Allow the output buffer to drain, discard pending input, and set the
/// current serial port settings.
pub const TCSETSF: c_ulong = 0x5404;

/// Equivalent to `tcsendbreak(fd, arg)`.
pub const TCSBRK: c_ulong = 0x5409;
/// Equivalent to `tcflow(fd, arg)`.
pub const TCXONC: c_ulong = 0x540A;
/// Equivalent to `tcflush(fd, arg)`.
pub const TCFLSH: c_ulong = 0x540B;

/// Make the given terminal the controlling terminal of the calling process.
pub const TIOCSCTTY: c_ulong = 0x540E;
/// When successful, equivalent to `*argp = tcgetpgrp(fd)`.
/// Get the process group ID of the foreground process group on this terminal.
pub const TIOCGPGRP: c_ulong = 0x540F;
/// Equivalent to `tcsetpgrp(fd, *argp)`.
pub const TIOCSPGRP: c_ulong = 0x5410;

/// Get window size.
pub const TIOCGWINSZ: c_ulong = 0x5413;
/// Set window size.
pub const TIOCSWINSZ: c_ulong = 0x5414;

//TODO: used by tcgetsid, not implemented yet on redox
/// When successful, equivalent to `*argp = tcgetsid(fd)`.
/// Get the session ID of the given terminal.
pub const TIOCGSID: c_ulong = 0x5429;

/// Get the number of bytes in the input buffer.
pub const FIONREAD: c_ulong = 0x541B;

/// Not recommended, should use POSIX `O_NONBLOCK` instead.
pub const FIONBIO: c_ulong = 0x5421;

/// Set (if `*lock` is nonzero) or remove (if `*lock` is zero) the lock on
/// the pseudoterminal slave device.
pub const TIOCSPTLCK: c_ulong = 0x4004_5431;
/// Place the current lock state of the pseudoterminal slave device in the
/// location pointed to by `lock`.
pub const TIOCGPTLCK: c_ulong = 0x8004_5439;

/// POSIX `sockatmark()` from `sys/socket.h` is intended to replace this.
pub const SIOCATMARK: c_ulong = 0x8905;
