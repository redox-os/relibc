//! `crypt.h` implementation.
//!
//! Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/crypt.3.html>.

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

use ::scrypt::password_hash::{Salt, SaltString};
use alloc::{
    ffi::CString,
    string::{String, ToString},
};
use core::ptr;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

use crate::{
    c_str::CStr,
    header::{errno::EINVAL, stdlib::rand},
    platform::{self, types::*},
};

mod blowfish;
mod md5;
mod pbkdf2;
mod scrypt;
mod sha;

use self::{
    blowfish::crypt_blowfish,
    md5::crypt_md5,
    pbkdf2::crypt_pbkdf2,
    scrypt::crypt_scrypt,
    sha::{crypt_sha, ShaType::*},
};

/// See <https://www.man7.org/linux/man-pages/man3/crypt.3.html>.
#[repr(C)]
pub struct crypt_data {
    initialized: c_int,
    buff: [c_char; 256],
}

impl crypt_data {
    pub fn new() -> Self {
        crypt_data {
            initialized: 1,
            buff: [0; 256],
        }
    }
}

fn gen_salt() -> Option<String> {
    let mut rng = SmallRng::seed_from_u64(unsafe { rand() as u64 });
    let mut bytes = [0u8; Salt::RECOMMENDED_LENGTH];
    rng.fill_bytes(&mut bytes);
    Some(SaltString::encode_b64(&bytes).ok()?.as_str().to_string())
}

/// See <https://www.man7.org/linux/man-pages/man3/crypt.3.html>.
#[no_mangle]
pub unsafe extern "C" fn crypt_r(
    key: *const c_char,
    setting: *const c_char,
    data: *mut crypt_data,
) -> *mut c_char {
    if unsafe { (*data).initialized } == 0 {
        unsafe { *data = crypt_data::new() };
    }

    let key = unsafe { CStr::from_ptr(key) }
        .to_str()
        .expect("key must be utf-8");
    let setting = unsafe { CStr::from_ptr(setting) }
        .to_str()
        .expect("setting must be utf-8");
    let setting_bytes = setting.as_bytes();

    let encoded = if setting_bytes[0] == b'$' && setting_bytes[1] != 0 && setting_bytes[2] != 0 {
        if setting_bytes[1] == b'1' && setting_bytes[2] == b'$' {
            crypt_md5(key, setting)
        } else if setting_bytes[1] == b'2' && setting_bytes[3] == b'$' {
            crypt_blowfish(key, setting)
        } else if setting_bytes[1] == b'5' && setting_bytes[2] == b'$' {
            crypt_sha(key, setting, Sha256)
        } else if setting_bytes[1] == b'6' && setting_bytes[2] == b'$' {
            crypt_sha(key, setting, Sha512)
        } else if setting_bytes[1] == b'7' && setting_bytes[2] == b'$' {
            crypt_scrypt(key, setting)
        } else if setting_bytes[1] == b'8' && setting_bytes[2] == b'$' {
            crypt_pbkdf2(key, setting)
        } else {
            platform::ERRNO.set(EINVAL);
            return ptr::null_mut();
        }
    } else {
        None
    };

    if let Some(inner) = encoded {
        let len = inner.len();
        if let Ok(ret) = CString::new(inner) {
            let ret_ptr = ret.into_raw();
            let dst = unsafe { (*data).buff }.as_mut_ptr();
            unsafe {
                ptr::copy_nonoverlapping(ret_ptr, dst.cast(), len);
            }
            ret_ptr.cast()
        } else {
            ptr::null_mut()
        }
    } else {
        ptr::null_mut()
    }
}
