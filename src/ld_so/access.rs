use crate::{
    c_str::{CStr, CString},
    error::Errno,
    platform::{types::*, Pal, Sys},
};

pub fn accessible(path: &str, mode: c_int) -> Result<(), Errno> {
    let path_c = CString::new(path.as_bytes()).unwrap();
    unsafe { Sys::access(CStr::from_ptr(path_c.as_ptr()), mode) }
}
