//! `locale.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/locale.h.html>.

use alloc::{boxed::Box, ffi::CString, string::String};
use core::{ptr, str::FromStr};

use crate::{
    c_str::CStr,
    error::{Errno, ResultExtPtrMut},
    fs::File,
    header::{errno, fcntl},
    io::Read,
    platform::types::{c_char, c_int},
};

// Can't use &str because of the mutability
static mut C_LOCALE: [c_char; 2] = [b'C' as c_char, 0];

mod constants;
use constants::*;
mod data;
use data::*;

use super::bits_locale_t::locale_t;
/// constant struct to "C" or "POSIX" locale
/// mutable because POSIX demands a mutable pointer
static mut POSIX_LOCALE: lconv = posix_lconv();
pub const LC_GLOBAL_LOCALE: locale_t = -1isize as locale_t;
/// process-wide locale, used by setlocale() and localeconv()
static mut GLOBAL_LOCALE: *mut GlobalLocaleData = ptr::null_mut();
/// thread-wide locale, used by uselocale() and localeconv()
#[thread_local]
pub(crate) static mut THREAD_LOCALE: *mut LocaleData = ptr::null_mut();

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/localeconv.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn localeconv() -> *mut lconv {
    let current = unsafe { uselocale(ptr::null_mut()) };
    if current == LC_GLOBAL_LOCALE || current.is_null() {
        if !unsafe { GLOBAL_LOCALE.is_null() } {
            // safety: GLOBAL_LOCALE is never set to null again
            unsafe { &raw mut (*GLOBAL_LOCALE).data.lconv }
        } else {
            &raw mut POSIX_LOCALE
        }
    } else {
        let current = current as *mut LocaleData;
        unsafe { &raw mut (*current).lconv }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setlocale.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setlocale(category: c_int, locale: *const c_char) -> *mut c_char {
    if unsafe { GLOBAL_LOCALE.is_null() } {
        let new_global = GlobalLocaleData::new();
        unsafe { GLOBAL_LOCALE = Box::into_raw(new_global) };
    };
    let Some(global) = (unsafe { GLOBAL_LOCALE.as_mut() }) else {
        return ptr::null_mut();
    };

    if locale.is_null() {
        let Some(name) = global.get_name(category) else {
            return ptr::null_mut();
        };
        return name.as_ptr() as *mut c_char;
    }

    let name = unsafe { CStr::from_ptr(locale).to_str().unwrap_or("C") };

    let locale_file = if name == "" || name == "C" || name == "POSIX" {
        // TODO: name == "" should read from LANG env
        Ok(LocaleData::posix())
    } else {
        load_locale_file(name)
    };

    match locale_file {
        Ok(loc_ptr) => {
            global.data.copy_category(&loc_ptr, category);
            let Some(name) = global.set_name(category, CString::from_str(name).unwrap()) else {
                return ptr::null_mut();
            };
            name.as_ptr() as *mut c_char
        }
        Err(_) => ptr::null_mut(),
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/uselocale.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uselocale(newloc: locale_t) -> locale_t {
    let old_loc = if unsafe { THREAD_LOCALE.is_null() } {
        LC_GLOBAL_LOCALE
    } else {
        (unsafe { THREAD_LOCALE }) as locale_t
    };

    if !newloc.is_null() {
        unsafe {
            THREAD_LOCALE = if newloc == LC_GLOBAL_LOCALE {
                ptr::null_mut()
            } else {
                newloc as *mut LocaleData
            }
        };
    }

    old_loc
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/newlocale.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn newlocale(mask: c_int, locale: *const c_char, base: locale_t) -> locale_t {
    let name = unsafe { CStr::from_ptr(locale) }
        .to_string_lossy()
        .into_owned();
    let name = name.as_str();
    let mut new_locale = if name == "" || name == "C" || name == "POSIX" {
        // TODO: name == "" should read from LANG env
        Ok(LocaleData::posix())
    } else {
        load_locale_file(name)
    };
    if base != LC_GLOBAL_LOCALE {
        // borrowing here
        let base = base as *const _ as *const LocaleData;
        if let Ok(new_locale) = new_locale.as_mut() {
            if let Some(base) = unsafe { base.as_ref() } {
                // copy old values if not containing the mask
                if (mask & LC_NUMERIC_MASK) == 0 {
                    new_locale.copy_category(base, LC_NUMERIC);
                }
                if (mask & LC_MONETARY_MASK) == 0 {
                    new_locale.copy_category(base, LC_MONETARY);
                }
                // TODO: other categories?
            }
        }
    }
    new_locale.or_errno_null_mut() as *mut _
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/freelocale.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn freelocale(loc: locale_t) {
    if !loc.is_null() && loc != LC_GLOBAL_LOCALE {
        drop(unsafe { Box::from_raw(loc as *mut LocaleData) });
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/duplocale.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn duplocale(loc: locale_t) -> locale_t {
    if loc.is_null() {
        // TODO: errno?
        loc
    } else if loc == LC_GLOBAL_LOCALE {
        Box::into_raw(LocaleData::posix()) as locale_t
    } else {
        // borrowing here
        let loc = loc as *const _ as *const LocaleData;
        Box::into_raw(unsafe { Box::from((*loc).clone()) }) as locale_t
    }
}

pub fn load_locale_file(name: &str) -> Result<Box<LocaleData>, Errno> {
    let mut path = String::from("/usr/share/i18n/locales/");
    path.push_str(name);

    let path_c = CString::new(path).map_err(|_| Errno(errno::EINVAL))?;
    let mut content = String::new();

    let mut file = File::open(path_c.as_c_str().into(), fcntl::O_RDONLY)?;
    file.read_to_string(&mut content)
        .map_err(|_| Errno(errno::EIO))?;

    let toml = PosixLocaleDef::parse(&content);
    Ok(LocaleData::new(CString::from_str(name).unwrap(), toml))
}
