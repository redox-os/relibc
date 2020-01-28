/* c_cc { */
pub const VEOF: usize = 0;
pub const VEOL: usize = 1;
pub const VEOL2: usize = 2;
pub const VERASE: usize = 3;
pub const VWERASE: usize = 4;
pub const VKILL: usize = 5;
pub const VREPRINT: usize = 6;
pub const VSWTC: usize = 7;
pub const VINTR: usize = 8;
pub const VQUIT: usize = 9;
pub const VSUSP: usize = 10;
pub const VSTART: usize = 12;
pub const VSTOP: usize = 13;
pub const VLNEXT: usize = 14;
pub const VDISCARD: usize = 15;
pub const VMIN: usize = 16;
pub const VTIME: usize = 17;
pub const NCCS: usize = 32;
/* } c_cc */

/* c_iflag { */
pub const IGNBRK: usize = 0o000_001;
pub const BRKINT: usize = 0o000_002;
pub const IGNPAR: usize = 0o000_004;
pub const PARMRK: usize = 0o000_010;
pub const INPCK: usize = 0o000_020;
pub const ISTRIP: usize = 0o000_040;
pub const INLCR: usize = 0o000_100;
pub const IGNCR: usize = 0o000_200;
pub const ICRNL: usize = 0o000_400;
pub const IXON: usize = 0o001_000;
pub const IXOFF: usize = 0o002_000;
/* } c_iflag */

/* c_oflag { */
pub const OPOST: usize = 0o000_001;
pub const ONLCR: usize = 0o000_002;
pub const OLCUC: usize = 0o000_004;
pub const OCRNL: usize = 0o000_010;
pub const ONOCR: usize = 0o000_020;
pub const ONLRET: usize = 0o000_040;
pub const OFILL: usize = 0o0000_100;
pub const OFDEL: usize = 0o0000_200;
/* } c_oflag */

/* c_cflag { */
pub const B0: usize = 0o000_000;
pub const B50: usize = 0o000_001;
pub const B75: usize = 0o000_002;
pub const B110: usize = 0o000_003;
pub const B134: usize = 0o000_004;
pub const B150: usize = 0o000_005;
pub const B200: usize = 0o000_006;
pub const B300: usize = 0o000_007;
pub const B600: usize = 0o000_010;
pub const B1200: usize = 0o000_011;
pub const B1800: usize = 0o000_012;
pub const B2400: usize = 0o000_013;
pub const B4800: usize = 0o000_014;
pub const B9600: usize = 0o000_015;
pub const B19200: usize = 0o000_016;
pub const B38400: usize = 0o000_017;

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

pub const CSIZE: usize = 0o001_400;
pub const CS5: usize = 0o000_000;
pub const CS6: usize = 0o000_400;
pub const CS7: usize = 0o001_000;
pub const CS8: usize = 0o001_400;

pub const CSTOPB: usize = 0o002_000;
pub const CREAD: usize = 0o004_000;
pub const PARENB: usize = 0o010_000;
pub const PARODD: usize = 0o020_000;
pub const HUPCL: usize = 0o040_000;

pub const CLOCAL: usize = 0o0100000;
/* } c_clfag */

/* c_lflag { */
pub const ISIG: usize = 0x0000_0080;
pub const ICANON: usize = 0x0000_0100;
pub const ECHO: usize = 0x0000_0008;
pub const ECHOE: usize = 0x0000_0002;
pub const ECHOK: usize = 0x0000_0004;
pub const ECHONL: usize = 0x0000_0010;
pub const NOFLSH: usize = 0x8000_0000;
pub const TOSTOP: usize = 0x0040_0000;
pub const IEXTEN: usize = 0x0000_0400;
/* } c_lflag */
