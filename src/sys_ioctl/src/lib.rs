//! ioctl implementation for linux

#![no_std]

#[cfg(target_os = "linux")]
pub mod inner {
    extern crate platform;

    use self::platform::types::*;

    #[repr(C)]
    pub struct winsize {
        ws_row: c_ushort,
        ws_col: c_ushort,
        ws_xpixel: c_ushort,
        ws_ypixel: c_ushort
    }

    #[no_mangle]
    pub extern "C" fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> c_int {
        // TODO: Somehow support varargs to syscall??
        platform::ioctl(fd, request, out)
    }

    pub const TCGETS: u32 = 0x5401;
    pub const TCSETS: u32 = 0x5402;
    pub const TCSETSW: u32 = 0x5403;
    pub const TCSETSF: u32 = 0x5404;
    pub const TCGETA: u32 = 0x5405;
    pub const TCSETA: u32 = 0x5406;
    pub const TCSETAW: u32 = 0x5407;
    pub const TCSETAF: u32 = 0x5408;
    pub const TCSBRK: u32 = 0x5409;
    pub const TCXONC: u32 = 0x540A;
    pub const TCFLSH: u32 = 0x540B;
    pub const TIOCEXCL: u32 = 0x540C;
    pub const TIOCNXCL: u32 = 0x540D;
    pub const TIOCSCTTY: u32 = 0x540E;
    pub const TIOCGPGRP: u32 = 0x540F;
    pub const TIOCSPGRP: u32 = 0x5410;
    pub const TIOCOUTQ: u32 = 0x5411;
    pub const TIOCSTI: u32 = 0x5412;
    pub const TIOCGWINSZ: u32 = 0x5413;
    pub const TIOCSWINSZ: u32 = 0x5414;
    pub const TIOCMGET: u32 = 0x5415;
    pub const TIOCMBIS: u32 = 0x5416;
    pub const TIOCMBIC: u32 = 0x5417;
    pub const TIOCMSET: u32 = 0x5418;
    pub const TIOCGSOFTCAR: u32 = 0x5419;
    pub const TIOCSSOFTCAR: u32 = 0x541A;
    pub const FIONREAD: u32 = 0x541B;
    pub const TIOCINQ: u32 = FIONREAD;
    pub const TIOCLINUX: u32 = 0x541C;
    pub const TIOCCONS: u32 = 0x541D;
    pub const TIOCGSERIAL: u32 = 0x541E;
    pub const TIOCSSERIAL: u32 = 0x541F;
    pub const TIOCPKT: u32 = 0x5420;
    pub const FIONBIO: u32 = 0x5421;
    pub const TIOCNOTTY: u32 = 0x5422;
    pub const TIOCSETD: u32 = 0x5423;
    pub const TIOCGETD: u32 = 0x5424;
    pub const TCSBRKP: u32 = 0x5425;
    pub const TIOCSBRK: u32 = 0x5427;
    pub const TIOCCBRK: u32 = 0x5428;
    pub const TIOCGSID: u32 = 0x5429;
    pub const TIOCGRS485: u32 = 0x542E;
    pub const TIOCSRS485: u32 = 0x542F;
    pub const TIOCGPTN: u32 = 0x80045430;
    pub const TIOCSPTLCK: u32 = 0x40045431;
    pub const TIOCGDEV: u32 = 0x80045432;
    pub const TCGETX: u32 = 0x5432;
    pub const TCSETX: u32 = 0x5433;
    pub const TCSETXF: u32 = 0x5434;
    pub const TCSETXW: u32 = 0x5435;
    pub const TIOCSIG: u32 = 0x40045436;
    pub const TIOCVHANGUP: u32 = 0x5437;
    pub const TIOCGPKT: u32 = 0x80045438;
    pub const TIOCGPTLCK: u32 = 0x80045439;
    pub const TIOCGEXCL: u32 = 0x80045440;
    pub const TIOCGPTPEER: u32 = 0x5441;

    pub const FIONCLEX: u32 = 0x5450;
    pub const FIOCLEX: u32 = 0x5451;
    pub const FIOASYNC: u32 = 0x5452;
    pub const TIOCSERCONFIG: u32 = 0x5453;
    pub const TIOCSERGWILD: u32 = 0x5454;
    pub const TIOCSERSWILD: u32 = 0x5455;
    pub const TIOCGLCKTRMIOS: u32 = 0x5456;
    pub const TIOCSLCKTRMIOS: u32 = 0x5457;
    pub const TIOCSERGSTRUCT: u32 = 0x5458;
    pub const TIOCSERGETLSR: u32 = 0x5459;
    pub const TIOCSERGETMULTI: u32 = 0x545A;
    pub const TIOCSERSETMULTI: u32 = 0x545B;

    pub const TIOCMIWAIT: u32 = 0x545C;
    pub const TIOCGICOUNT: u32 = 0x545D;
    pub const FIOQSIZE: u32 = 0x5460;

    pub const TIOCPKT_DATA: u32 = 0;
    pub const TIOCPKT_FLUSHREAD: u32 = 1;
    pub const TIOCPKT_FLUSHWRITE: u32 = 2;
    pub const TIOCPKT_STOP: u32 = 4;
    pub const TIOCPKT_START: u32 = 8;
    pub const TIOCPKT_NOSTOP: u32 = 16;
    pub const TIOCPKT_DOSTOP: u32 = 32;
    pub const TIOCPKT_IOCTL: u32 = 64;

    pub const TIOCSER_TEMT: u32 = 0x01;

    pub const TIOCM_LE: u32 = 0x001;
    pub const TIOCM_DTR: u32 = 0x002;
    pub const TIOCM_RTS: u32 = 0x004;
    pub const TIOCM_ST: u32 = 0x008;
    pub const TIOCM_SR: u32 = 0x010;
    pub const TIOCM_CTS: u32 = 0x020;
    pub const TIOCM_CAR: u32 = 0x040;
    pub const TIOCM_RNG: u32 = 0x080;
    pub const TIOCM_DSR: u32 = 0x100;
    pub const TIOCM_CD: u32 = TIOCM_CAR;
    pub const TIOCM_RI: u32 = TIOCM_RNG;
    pub const TIOCM_OUT1: u32 = 0x2000;
    pub const TIOCM_OUT2: u32 = 0x4000;
    pub const TIOCM_LOOP: u32 = 0x8000;

    pub const N_TTY: u32 = 0;
    pub const N_SLIP: u32 = 1;
    pub const N_MOUSE: u32 = 2;
    pub const N_PPP: u32 = 3;
    pub const N_STRIP: u32 = 4;
    pub const N_AX25: u32 = 5;
    pub const N_X25: u32 = 6;
    pub const N_6PACK: u32 = 7;
    pub const N_MASC: u32 = 8;
    pub const N_R3964: u32 = 9;
    pub const N_PROFIBUS_FDL: u32 = 10;
    pub const N_IRDA: u32 = 11;
    pub const N_SMSBLOCK: u32 = 12;
    pub const N_HDLC: u32 = 13;
    pub const N_SYNC_PPP: u32 = 14;
    pub const N_HCI: u32 = 15;

    pub const FIOSETOWN: u32 = 0x8901;
    pub const SIOCSPGRP: u32 = 0x8902;
    pub const FIOGETOWN: u32 = 0x8903;
    pub const SIOCGPGRP: u32 = 0x8904;
    pub const SIOCATMARK: u32 = 0x8905;
    pub const SIOCGSTAMP: u32 = 0x8906;
    pub const SIOCGSTAMPNS: u32 = 0x8907;

    pub const SIOCADDRT: u32 = 0x890B;
    pub const SIOCDELRT: u32 = 0x890C;
    pub const SIOCRTMSG: u32 = 0x890D;

    pub const SIOCGIFNAME: u32 = 0x8910;
    pub const SIOCSIFLINK: u32 = 0x8911;
    pub const SIOCGIFCONF: u32 = 0x8912;
    pub const SIOCGIFFLAGS: u32 = 0x8913;
    pub const SIOCSIFFLAGS: u32 = 0x8914;
    pub const SIOCGIFADDR: u32 = 0x8915;
    pub const SIOCSIFADDR: u32 = 0x8916;
    pub const SIOCGIFDSTADDR: u32 = 0x8917;
    pub const SIOCSIFDSTADDR: u32 = 0x8918;
    pub const SIOCGIFBRDADDR: u32 = 0x8919;
    pub const SIOCSIFBRDADDR: u32 = 0x891a;
    pub const SIOCGIFNETMASK: u32 = 0x891b;
    pub const SIOCSIFNETMASK: u32 = 0x891c;
    pub const SIOCGIFMETRIC: u32 = 0x891d;
    pub const SIOCSIFMETRIC: u32 = 0x891e;
    pub const SIOCGIFMEM: u32 = 0x891f;
    pub const SIOCSIFMEM: u32 = 0x8920;
    pub const SIOCGIFMTU: u32 = 0x8921;
    pub const SIOCSIFMTU: u32 = 0x8922;
    pub const SIOCSIFNAME: u32 = 0x8923;
    pub const SIOCSIFHWADDR: u32 = 0x8924;
    pub const SIOCGIFENCAP: u32 = 0x8925;
    pub const SIOCSIFENCAP: u32 = 0x8926;
    pub const SIOCGIFHWADDR: u32 = 0x8927;
    pub const SIOCGIFSLAVE: u32 = 0x8929;
    pub const SIOCSIFSLAVE: u32 = 0x8930;
    pub const SIOCADDMULTI: u32 = 0x8931;
    pub const SIOCDELMULTI: u32 = 0x8932;
    pub const SIOCGIFINDEX: u32 = 0x8933;
    pub const SIOGIFINDEX: u32 = SIOCGIFINDEX;
    pub const SIOCSIFPFLAGS: u32 = 0x8934;
    pub const SIOCGIFPFLAGS: u32 = 0x8935;
    pub const SIOCDIFADDR: u32 = 0x8936;
    pub const SIOCSIFHWBROADCAST: u32 = 0x8937;
    pub const SIOCGIFCOUNT: u32 = 0x8938;

    pub const SIOCGIFBR: u32 = 0x8940;
    pub const SIOCSIFBR: u32 = 0x8941;

    pub const SIOCGIFTXQLEN: u32 = 0x8942;
    pub const SIOCSIFTXQLEN: u32 = 0x8943;

    pub const SIOCDARP: u32 = 0x8953;
    pub const SIOCGARP: u32 = 0x8954;
    pub const SIOCSARP: u32 = 0x8955;

    pub const SIOCDRARP: u32 = 0x8960;
    pub const SIOCGRARP: u32 = 0x8961;
    pub const SIOCSRARP: u32 = 0x8962;

    pub const SIOCGIFMAP: u32 = 0x8970;
    pub const SIOCSIFMAP: u32 = 0x8971;

    pub const SIOCADDDLCI: u32 = 0x8980;
    pub const SIOCDELDLCI: u32 = 0x8981;

    pub const SIOCDEVPRIVATE: u32 = 0x89F0;
    pub const SIOCPROTOPRIVATE: u32 = 0x89E0;
}

#[cfg(target_os = "linux")]
pub use inner::*;
