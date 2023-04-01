use crate::platform::{types::*, Sys};

#[no_mangle]
pub unsafe extern "C" fn ioctl(fd: c_int, request: c_ulong, out: *mut c_void) -> c_int {
    // TODO: Somehow support varargs to syscall??
    Sys::ioctl(fd, request, out)
}

pub const TCGETS: c_ulong = 0x5401;
pub const TCSETS: c_ulong = 0x5402;
pub const TCSETSW: c_ulong = 0x5403;
pub const TCSETSF: c_ulong = 0x5404;
pub const TCGETA: c_ulong = 0x5405;
pub const TCSETA: c_ulong = 0x5406;
pub const TCSETAW: c_ulong = 0x5407;
pub const TCSETAF: c_ulong = 0x5408;
pub const TCSBRK: c_ulong = 0x5409;
pub const TCXONC: c_ulong = 0x540A;
pub const TCFLSH: c_ulong = 0x540B;
pub const TIOCEXCL: c_ulong = 0x540C;
pub const TIOCNXCL: c_ulong = 0x540D;
pub const TIOCSCTTY: c_ulong = 0x540E;
pub const TIOCGPGRP: c_ulong = 0x540F;
pub const TIOCSPGRP: c_ulong = 0x5410;
pub const TIOCOUTQ: c_ulong = 0x5411;
pub const TIOCSTI: c_ulong = 0x5412;
pub const TIOCGWINSZ: c_ulong = 0x5413;
pub const TIOCSWINSZ: c_ulong = 0x5414;
pub const TIOCMGET: c_ulong = 0x5415;
pub const TIOCMBIS: c_ulong = 0x5416;
pub const TIOCMBIC: c_ulong = 0x5417;
pub const TIOCMSET: c_ulong = 0x5418;
pub const TIOCGSOFTCAR: c_ulong = 0x5419;
pub const TIOCSSOFTCAR: c_ulong = 0x541A;
pub const FIONREAD: c_ulong = 0x541B;
pub const TIOCINQ: c_ulong = FIONREAD;
pub const TIOCLINUX: c_ulong = 0x541C;
pub const TIOCCONS: c_ulong = 0x541D;
pub const TIOCGSERIAL: c_ulong = 0x541E;
pub const TIOCSSERIAL: c_ulong = 0x541F;
pub const TIOCPKT: c_ulong = 0x5420;
pub const FIONBIO: c_ulong = 0x5421;
pub const TIOCNOTTY: c_ulong = 0x5422;
pub const TIOCSETD: c_ulong = 0x5423;
pub const TIOCGETD: c_ulong = 0x5424;
pub const TCSBRKP: c_ulong = 0x5425;
pub const TIOCSBRK: c_ulong = 0x5427;
pub const TIOCCBRK: c_ulong = 0x5428;
pub const TIOCGSID: c_ulong = 0x5429;
pub const TIOCGRS485: c_ulong = 0x542E;
pub const TIOCSRS485: c_ulong = 0x542F;
pub const TIOCGPTN: c_ulong = 0x8004_5430;
pub const TIOCSPTLCK: c_ulong = 0x4004_5431;
pub const TIOCGDEV: c_ulong = 0x8004_5432;
pub const TCGETX: c_ulong = 0x5432;
pub const TCSETX: c_ulong = 0x5433;
pub const TCSETXF: c_ulong = 0x5434;
pub const TCSETXW: c_ulong = 0x5435;
pub const TIOCSIG: c_ulong = 0x4004_5436;
pub const TIOCVHANGUP: c_ulong = 0x5437;
pub const TIOCGPKT: c_ulong = 0x8004_5438;
pub const TIOCGPTLCK: c_ulong = 0x8004_5439;
pub const TIOCGEXCL: c_ulong = 0x8004_5440;
pub const TIOCGPTPEER: c_ulong = 0x5441;

pub const FIONCLEX: c_ulong = 0x5450;
pub const FIOCLEX: c_ulong = 0x5451;
pub const FIOASYNC: c_ulong = 0x5452;
pub const TIOCSERCONFIG: c_ulong = 0x5453;
pub const TIOCSERGWILD: c_ulong = 0x5454;
pub const TIOCSERSWILD: c_ulong = 0x5455;
pub const TIOCGLCKTRMIOS: c_ulong = 0x5456;
pub const TIOCSLCKTRMIOS: c_ulong = 0x5457;
pub const TIOCSERGSTRUCT: c_ulong = 0x5458;
pub const TIOCSERGETLSR: c_ulong = 0x5459;
pub const TIOCSERGETMULTI: c_ulong = 0x545A;
pub const TIOCSERSETMULTI: c_ulong = 0x545B;

pub const TIOCMIWAIT: c_ulong = 0x545C;
pub const TIOCGICOUNT: c_ulong = 0x545D;
pub const FIOQSIZE: c_ulong = 0x5460;

pub const TIOCPKT_DATA: c_ulong = 0;
pub const TIOCPKT_FLUSHREAD: c_ulong = 1;
pub const TIOCPKT_FLUSHWRITE: c_ulong = 2;
pub const TIOCPKT_STOP: c_ulong = 4;
pub const TIOCPKT_START: c_ulong = 8;
pub const TIOCPKT_NOSTOP: c_ulong = 16;
pub const TIOCPKT_DOSTOP: c_ulong = 32;
pub const TIOCPKT_IOCTL: c_ulong = 64;

pub const TIOCSER_TEMT: c_ulong = 0x01;

pub const TIOCM_LE: c_ulong = 0x001;
pub const TIOCM_DTR: c_ulong = 0x002;
pub const TIOCM_RTS: c_ulong = 0x004;
pub const TIOCM_ST: c_ulong = 0x008;
pub const TIOCM_SR: c_ulong = 0x010;
pub const TIOCM_CTS: c_ulong = 0x020;
pub const TIOCM_CAR: c_ulong = 0x040;
pub const TIOCM_RNG: c_ulong = 0x080;
pub const TIOCM_DSR: c_ulong = 0x100;
pub const TIOCM_CD: c_ulong = TIOCM_CAR;
pub const TIOCM_RI: c_ulong = TIOCM_RNG;
pub const TIOCM_OUT1: c_ulong = 0x2000;
pub const TIOCM_OUT2: c_ulong = 0x4000;
pub const TIOCM_LOOP: c_ulong = 0x8000;

pub const N_TTY: c_ulong = 0;
pub const N_SLIP: c_ulong = 1;
pub const N_MOUSE: c_ulong = 2;
pub const N_PPP: c_ulong = 3;
pub const N_STRIP: c_ulong = 4;
pub const N_AX25: c_ulong = 5;
pub const N_X25: c_ulong = 6;
pub const N_6PACK: c_ulong = 7;
pub const N_MASC: c_ulong = 8;
pub const N_R3964: c_ulong = 9;
pub const N_PROFIBUS_FDL: c_ulong = 10;
pub const N_IRDA: c_ulong = 11;
pub const N_SMSBLOCK: c_ulong = 12;
pub const N_HDLC: c_ulong = 13;
pub const N_SYNC_PPP: c_ulong = 14;
pub const N_HCI: c_ulong = 15;

pub const FIOSETOWN: c_ulong = 0x8901;
pub const SIOCSPGRP: c_ulong = 0x8902;
pub const FIOGETOWN: c_ulong = 0x8903;
pub const SIOCGPGRP: c_ulong = 0x8904;
pub const SIOCATMARK: c_ulong = 0x8905;
pub const SIOCGSTAMP: c_ulong = 0x8906;
pub const SIOCGSTAMPNS: c_ulong = 0x8907;

pub const SIOCADDRT: c_ulong = 0x890B;
pub const SIOCDELRT: c_ulong = 0x890C;
pub const SIOCRTMSG: c_ulong = 0x890D;

pub const SIOCGIFNAME: c_ulong = 0x8910;
pub const SIOCSIFLINK: c_ulong = 0x8911;
pub const SIOCGIFCONF: c_ulong = 0x8912;
pub const SIOCGIFFLAGS: c_ulong = 0x8913;
pub const SIOCSIFFLAGS: c_ulong = 0x8914;
pub const SIOCGIFADDR: c_ulong = 0x8915;
pub const SIOCSIFADDR: c_ulong = 0x8916;
pub const SIOCGIFDSTADDR: c_ulong = 0x8917;
pub const SIOCSIFDSTADDR: c_ulong = 0x8918;
pub const SIOCGIFBRDADDR: c_ulong = 0x8919;
pub const SIOCSIFBRDADDR: c_ulong = 0x891a;
pub const SIOCGIFNETMASK: c_ulong = 0x891b;
pub const SIOCSIFNETMASK: c_ulong = 0x891c;
pub const SIOCGIFMETRIC: c_ulong = 0x891d;
pub const SIOCSIFMETRIC: c_ulong = 0x891e;
pub const SIOCGIFMEM: c_ulong = 0x891f;
pub const SIOCSIFMEM: c_ulong = 0x8920;
pub const SIOCGIFMTU: c_ulong = 0x8921;
pub const SIOCSIFMTU: c_ulong = 0x8922;
pub const SIOCSIFNAME: c_ulong = 0x8923;
pub const SIOCSIFHWADDR: c_ulong = 0x8924;
pub const SIOCGIFENCAP: c_ulong = 0x8925;
pub const SIOCSIFENCAP: c_ulong = 0x8926;
pub const SIOCGIFHWADDR: c_ulong = 0x8927;
pub const SIOCGIFSLAVE: c_ulong = 0x8929;
pub const SIOCSIFSLAVE: c_ulong = 0x8930;
pub const SIOCADDMULTI: c_ulong = 0x8931;
pub const SIOCDELMULTI: c_ulong = 0x8932;
pub const SIOCGIFINDEX: c_ulong = 0x8933;
pub const SIOGIFINDEX: c_ulong = SIOCGIFINDEX;
pub const SIOCSIFPFLAGS: c_ulong = 0x8934;
pub const SIOCGIFPFLAGS: c_ulong = 0x8935;
pub const SIOCDIFADDR: c_ulong = 0x8936;
pub const SIOCSIFHWBROADCAST: c_ulong = 0x8937;
pub const SIOCGIFCOUNT: c_ulong = 0x8938;

pub const SIOCGIFBR: c_ulong = 0x8940;
pub const SIOCSIFBR: c_ulong = 0x8941;

pub const SIOCGIFTXQLEN: c_ulong = 0x8942;
pub const SIOCSIFTXQLEN: c_ulong = 0x8943;

pub const SIOCDARP: c_ulong = 0x8953;
pub const SIOCGARP: c_ulong = 0x8954;
pub const SIOCSARP: c_ulong = 0x8955;

pub const SIOCDRARP: c_ulong = 0x8960;
pub const SIOCGRARP: c_ulong = 0x8961;
pub const SIOCSRARP: c_ulong = 0x8962;

pub const SIOCGIFMAP: c_ulong = 0x8970;
pub const SIOCSIFMAP: c_ulong = 0x8971;

pub const SIOCADDDLCI: c_ulong = 0x8980;
pub const SIOCDELDLCI: c_ulong = 0x8981;

pub const SIOCDEVPRIVATE: c_ulong = 0x89F0;
pub const SIOCPROTOPRIVATE: c_ulong = 0x89E0;
