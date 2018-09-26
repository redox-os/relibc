use platform::types::*;

pub const EOF: c_int = -1;
pub const BUFSIZ: c_int = 1024;

pub const UNGET: c_int = 8;

pub const FILENAME_MAX: c_int = 4096;

pub const F_PERM: c_int = 1;
pub const F_NORD: c_int = 4;
pub const F_NOWR: c_int = 8;
pub const F_EOF: c_int = 16;
pub const F_ERR: c_int = 32;
pub const F_SVB: c_int = 64;
pub const F_APP: c_int = 128;
pub const F_BADJ: c_int = 256;

pub const SEEK_SET: c_int = 0;
pub const SEEK_CUR: c_int = 1;
pub const SEEK_END: c_int = 2;

pub const _IOFBF: c_int = 0;
pub const _IOLBF: c_int = 1;
pub const _IONBF: c_int = 2;

#[allow(non_camel_case_types)]
pub type fpos_t = off_t;
