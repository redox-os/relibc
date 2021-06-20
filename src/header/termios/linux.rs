use super::{tcflag_t, cc_t, speed_t};

/* c_cc { */
pub const VINTR: cc_t = 0;
pub const VQUIT: cc_t = 1;
pub const VERASE: cc_t = 2;
pub const VKILL: cc_t = 3;
pub const VEOF: cc_t = 4;
pub const VTIME: cc_t = 5;
pub const VMIN: cc_t = 6;
pub const VSWTC: cc_t = 7;
pub const VSTART: cc_t = 8;
pub const VSTOP: cc_t = 9;
pub const VSUSP: cc_t = 10;
pub const VEOL: cc_t = 11;
pub const VREPRINT: cc_t = 12;
pub const VDISCARD: cc_t = 13;
pub const VWERASE: cc_t = 14;
pub const VLNEXT: cc_t = 15;
pub const VEOL2: cc_t = 16;
pub const NCCS: usize = 32;
/* } c_cc */

/* c_iflag { */
pub const IGNBRK: tcflag_t = 0o000_001;
pub const BRKINT: tcflag_t = 0o000_002;
pub const IGNPAR: tcflag_t = 0o000_004;
pub const PARMRK: tcflag_t = 0o000_010;
pub const INPCK: tcflag_t = 0o000_020;
pub const ISTRIP: tcflag_t = 0o000_040;
pub const INLCR: tcflag_t = 0o000_100;
pub const IGNCR: tcflag_t = 0o000_200;
pub const ICRNL: tcflag_t = 0o000_400;
pub const IUCLC: tcflag_t = 0o001_000;
pub const IXON: tcflag_t = 0o002_000;
pub const IXANY: tcflag_t = 0o004_000;
pub const IXOFF: tcflag_t = 0o010_000;
pub const IMAXBEL: tcflag_t = 0o020_000;
pub const IUTF8: tcflag_t = 0o040_000;
/* } c_iflag */

/* c_oflag { */
pub const OPOST: tcflag_t = 0o000_001;
pub const OLCUC: tcflag_t = 0o000_002;
pub const ONLCR: tcflag_t = 0o000_004;
pub const OCRNL: tcflag_t = 0o000_010;
pub const ONOCR: tcflag_t = 0o000_020;
pub const ONLRET: tcflag_t = 0o000_040;
pub const OFILL: tcflag_t = 0o000_100;
pub const OFDEL: tcflag_t = 0o000_200;

pub const VTDLY: tcflag_t = 0o040_000;
pub const VT0: tcflag_t = 0o000_000;
pub const VT1: tcflag_t = 0o040_000;
/* } c_oflag */

/* c_cflag { */
pub const B0: speed_t = 0o000_000;
pub const B50: speed_t = 0o000_001;
pub const B75: speed_t = 0o000_002;
pub const B110: speed_t = 0o000_003;
pub const B134: speed_t = 0o000_004;
pub const B150: speed_t = 0o000_005;
pub const B200: speed_t = 0o000_006;
pub const B300: speed_t = 0o000_007;
pub const B600: speed_t = 0o000_010;
pub const B1200: speed_t = 0o000_011;
pub const B1800: speed_t = 0o000_012;
pub const B2400: speed_t = 0o000_013;
pub const B4800: speed_t = 0o000_014;
pub const B9600: speed_t = 0o000_015;
pub const B19200: speed_t = 0o000_016;
pub const B38400: speed_t = 0o000_017;

pub const B57600: speed_t = 0o010_001;
pub const B115200: speed_t = 0o010_002;
pub const B230400: speed_t = 0o010_003;
pub const B460800: speed_t = 0o010_004;
pub const B500000: speed_t = 0o010_005;
pub const B576000: speed_t = 0o010_006;
pub const B921600: speed_t = 0o010_007;
pub const B1000000: speed_t = 0o010_010;
pub const B1152000: speed_t = 0o010_011;
pub const B1500000: speed_t = 0o010_012;
pub const B2000000: speed_t = 0o010_013;
pub const B2500000: speed_t = 0o010_014;
pub const B3000000: speed_t = 0o010_015;
pub const B3500000: speed_t = 0o010_016;
pub const B4000000: speed_t = 0o010_017;

pub const CSIZE: tcflag_t = 0o000_060;
pub const CS5: tcflag_t = 0o000_000;
pub const CS6: tcflag_t = 0o000_020;
pub const CS7: tcflag_t = 0o000_040;
pub const CS8: tcflag_t = 0o000_060;

pub const CSTOPB: tcflag_t = 0o000_100;
pub const CREAD: tcflag_t = 0o000_200;
pub const PARENB: tcflag_t = 0o000_400;
pub const PARODD: tcflag_t = 0o001_000;
pub const HUPCL: tcflag_t = 0o002_000;
pub const CLOCAL: tcflag_t = 0o004_000;
/* } c_clfag */

/* c_lflag { */
pub const ISIG: tcflag_t = 0o000_001;
pub const ICANON: tcflag_t = 0o000_002;
pub const ECHO: tcflag_t = 0o000_010;
pub const ECHOE: tcflag_t = 0o000_020;
pub const ECHOK: tcflag_t = 0o000_040;
pub const ECHONL: tcflag_t = 0o000_100;
pub const NOFLSH: tcflag_t = 0o000_200;
pub const TOSTOP: tcflag_t = 0o000_400;
pub const IEXTEN: tcflag_t = 0o100_000;
/* } c_lflag */
