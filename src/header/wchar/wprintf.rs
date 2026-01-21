// TODO: reuse more code with the thin printf impl
use crate::{
    c_str::{self, WStr},
    header::stdio::printf::inner_printf,
    io::Write,
};
use core::ffi::VaList;

use crate::platform::{self, types::*};

pub unsafe fn wprintf(w: impl Write, format: WStr, ap: VaList) -> c_int {
    unsafe { inner_printf::<c_str::Wide>(w, format, ap).unwrap_or(-1) }
}
