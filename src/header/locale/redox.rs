use core::str::FromStr;

use alloc::{boxed::Box, ffi::CString, string::String};

use crate::{
    c_str::CStr,
    error::Errno,
    fs::File,
    header::{
        errno, fcntl,
        locale::{
            data::{LocaleData, RawLocale},
            locale_t,
        },
    },
    io::Read,
};

pub fn load_locale_file(name: &str) -> Result<Box<LocaleData>, Errno> {
    let mut path = String::from("/etc/locale.d/");
    path.push_str(name);

    let path_c = CString::new(path).map_err(|_| Errno(errno::EINVAL))?;
    let mut content = String::new();

    let mut file = File::open(path_c.as_c_str().into(), fcntl::O_RDONLY)?;
    file.read_to_string(&mut content)
        .map_err(|_| Errno(errno::EIO))?;

    let toml = RawLocale::parse(&content);
    Ok(LocaleData::new(CString::from_str(name).unwrap(), toml))
}
