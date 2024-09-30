#[cfg(target_os = "redox")]
use crate::header::unistd::{F_OK, R_OK, W_OK, X_OK};
use crate::{
    c_str::{CStr, CString},
    error::Errno,
    platform::{types::*, Pal, Sys},
};

pub fn accessible(path: &str, mode: c_int) -> Result<(), Errno> {
    let path_c = CString::new(path.as_bytes()).unwrap(); /*.map_err(|err| {
                                                             Error::Malformed(format!("invalid path '{}': {}", path, err))
                                                         })?;*/
    unsafe { Sys::access(CStr::from_ptr(path_c.as_ptr()), mode) }
}
