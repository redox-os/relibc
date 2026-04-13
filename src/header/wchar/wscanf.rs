use crate::{
    c_str::{self, WStr},
    header::stdio::{reader::Reader, scanf::inner_scanf},
    platform::types::*,
};
use core::ffi::VaList as va_list;

pub unsafe fn scanf(r: Reader<'_, c_str::Wide>, format: WStr, ap: va_list) -> c_int {
    match unsafe { inner_scanf::<c_str::Wide>(r, format.into(), ap) } {
        Ok(n) => n,
        Err(n) => n,
    }
}
