/* c_cc { */
/// EOF character.
/// End of file. Causes the pending tty buffer to be sent to the waiting user
/// program without waiting for end-of-line.
/// Canonical mode only.
pub const VEOF: usize = 0;
/// EOL character.
/// Additional end-of-line character.
/// Canonical mode only.
pub const VEOL: usize = 1;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/termios.3.html>.
///
/// EOL2 character.
/// Another end-of-line character.
pub const VEOL2: usize = 2;
/// ERASE character.
/// Erases the previous not-yet-erased character, but does not erase past `EOF`
/// or beginning-of-line.
/// Canonical mode only.
pub const VERASE: usize = 3;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/termios.3.html>.
///
/// WERASE character.
/// Word erase.
pub const VWERASE: usize = 4;
/// KILL character.
/// Erases the input since the last `EOF` or beginning-of-line.
/// Canonical mode only.
pub const VKILL: usize = 5;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/termios.3.html>.
///
/// REPRINT character.
/// Reprints unread characters.
pub const VREPRINT: usize = 6;
/// INTR character.
/// Send a `SIGINT` signal.
/// Canonical and Non-Canonical mode.
pub const VINTR: usize = 8;
/// QUIT character.
/// Send a `SIGQUIT` signal.
/// Canonical and Non-Canonical mode.
pub const VQUIT: usize = 9;
/// START character.
/// Restarts output stopped by the STOP character.
/// Canonical and Non-Canonical mode.
pub const VSTART: usize = 12;
/// STOP character.
/// Stops the output until the START character is typed.
/// Canonical and Non-Canonical mode.
pub const VSTOP: usize = 13;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/termios.3.html>.
///
/// LNEXT character.
/// Literal next. Quotes the next input character, depriving it of a possible
/// special meaning.
pub const VLNEXT: usize = 14;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/termios.3.html>.
///
/// DISCARD character (not supported under Linux).
/// Toggles discarding pending output.
pub const VDISCARD: usize = 15;
/// MIN value.
/// Minimum number of characters for noncanonical read.
/// Non-Canonical mode only.
pub const VMIN: usize = 16;
/// TIME value.
/// Timeout in decideconds for noncanonical read.
/// Non-Canonical mode only.
pub const VTIME: usize = 17;
/* } c_cc */

/* c_iflag { */
/// Enable start/stop output control.
pub const IXON: usize = 0o001_000;
/// Enable start/stop input control.
pub const IXOFF: usize = 0o002_000;
/* } c_iflag */

/* c_oflag { */
/// Map NL to CR-NL on output.
pub const ONLCR: usize = 0o000_002;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/termios.3.html>.
///
/// Map lowercase characters to uppercase on output.
pub const OLCUC: usize = 0o000_004;
/* } c_oflag */

/* c_cflag { */
pub const B57600: usize = 0o0_020;
pub const B115200: usize = 0o0_021;
pub const B230400: usize = 0o0_022;
pub const B460800: usize = 0o0_023;
pub const B500000: usize = 0o0_024;
pub const B576000: usize = 0o0_025;
pub const B921600: usize = 0o0_026;
pub const B1000000: usize = 0o0_027;
pub const B1152000: usize = 0o0_030;
pub const B1500000: usize = 0o0_031;
pub const B2000000: usize = 0o0_032;
pub const B2500000: usize = 0o0_033;
pub const B3000000: usize = 0o0_034;
pub const B3500000: usize = 0o0_035;
pub const B4000000: usize = 0o0_036;

/// Character size.
pub const CSIZE: usize = 0o001_400;
/// 6 bits.
pub const CS6: usize = 0o000_400;
/// 7 bits.
pub const CS7: usize = 0o001_000;
/// 8 bits.
pub const CS8: usize = 0o001_400;

/// Send two stop bits, else one.
pub const CSTOPB: usize = 0o002_000;
/// Enable receiver.
pub const CREAD: usize = 0o004_000;
/// Parity enable.
pub const PARENB: usize = 0o010_000;
/// Odd parity, else even.
pub const PARODD: usize = 0o020_000;
/// Hang up on last close.
pub const HUPCL: usize = 0o040_000;
/// Ignore modem status lines.
pub const CLOCAL: usize = 0o0100000;
/* } c_clfag */

/* c_lflag { */
/// Enable signals.
pub const ISIG: usize = 0x0000_0080;
/// Canoncal input (erase and kill processing).
pub const ICANON: usize = 0x0000_0100;
/// Echo erase character as error-correcting backspace.
pub const ECHOE: usize = 0x0000_0002;
/// Echo KILL.
pub const ECHOK: usize = 0x0000_0004;
/// Echo NL.
pub const ECHONL: usize = 0x0000_0010;
/// Disable flush after interrupt or quit.
pub const NOFLSH: usize = 0x8000_0000;
/// Send `SIGTTOU` for background output.
pub const TOSTOP: usize = 0x0040_0000;
/// Enable extended input character processing.
pub const IEXTEN: usize = 0x0000_0400;
/* } c_lflag */
