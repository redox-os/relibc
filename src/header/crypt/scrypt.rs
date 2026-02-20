use super::gen_salt;
use crate::platform::types::{c_uchar, c_uint};
use alloc::string::{String, ToString};
use base64ct::{Base64Bcrypt, Encoding};
use core::str;
use scrypt::{Params, scrypt};

/// Map for encoding and decoding
#[inline(always)]
fn to_digit(c: char, radix: u32) -> Option<u32> {
    match c {
        '.' => Some(0),
        '/' => Some(1),
        _ => c.to_digit(radix).map(|d| d + 2),
    }
}

/// Decodes a 5 character lengt str value to c_uint
///
/// # Arguments
///
/// * `value` - A string slice that represents a u32 value in base64
///
/// # Returns
///
/// * `Option<c_uint>` - Returns the decoded c_uint value if successful, otherwise None
fn dencode_uint(value: &str) -> Option<c_uint> {
    if value.len() != 5 {
        return None;
    }

    value
        .chars()
        .enumerate()
        .try_fold(0 as c_uint, |acc, (i, c)| {
            acc.checked_add((to_digit(c, 30)? as c_uint) << (i * 6))
        })
}

/// Reads settings for password encryption
///
/// # Arguments
///
/// * `setting` - A string slice that represents the settings
///
/// # Returns
///
/// * `Option<(c_uchar, c_uint, c_uint, String)>` - Returns a tuple containing the settings if successful, otherwise None
fn read_setting(setting: &str) -> Option<(c_uchar, c_uint, c_uint, String)> {
    let nlog2 = to_digit(setting.chars().next()?, 30)? as c_uchar;
    let r = dencode_uint(&setting[1..6])?;
    let p = dencode_uint(&setting[6..11])?;

    let salt = &setting[11..];
    let actual_salt = if !salt.is_empty() {
        salt.to_string()
    } else {
        gen_salt()?
    };

    Some((nlog2, r, p, actual_salt))
}

/// Performs Scrypt key derivation on a given password with a specific setting.
///
/// # Parameters
/// * `passw`: The password to be hashed. It must be a string slice (`&str`).
/// * `setting`: The settings for the Scrypt key derivation. It must be a string slice (`&str`)
///   and should follow the format `$<Nlog2>$<r>$<p>$<salt>`. The `<Nlog2>` part should be a decimal
///   number representing the logarithm base 2 of the CPU/memory cost factor N for Scrypt. The `<r>`
///   part should be a decimal number representing the block size r. The `<p>` part should be a decimal
///   number representing the parallelization factor p. The `<salt>` part should be a base64-encoded
///   string representing the salt to be used for the Scrypt function.
///
/// # Returns
/// * `Option<String>`: Returns `Some(String)` if the Scrypt operation was successful, where the
///   returned string is the result of the Scrypt operation formatted according to the Modular
///   Crypt Format (MCF). If the Scrypt operation failed, it returns `None`.
///
/// # Errors
/// * If the `setting` length is less than 14 characters.
/// * If the `scrypt` function fails to perform the Scrypt operation.
///
/// # Example
/// ```
/// let password = "my_password";
/// let setting = "$7$C6..../....SodiumChloride";
/// let result = crypt_scrypt(password, setting);
/// assert!(result.is_some());
/// ```
///
/// # Note
/// The `crypt_scrypt` function uses the Scrypt key derivation function for hashing.
/// The output of the Scrypt operation is base64-encoded using the BCrypt variant of base64.
pub fn crypt_scrypt(passw: &str, setting: &str) -> Option<String> {
    if setting.len() < 14 {
        return None;
    }

    let (nlog2, r, p, salt) = read_setting(&setting[3..])?;

    let params = Params::new(nlog2, r, p, 32).ok()?;
    let mut output = [0u8; 32];

    scrypt(passw.as_bytes(), salt.as_bytes(), &params, &mut output).ok()?;

    Some(format!(
        "$7${}${}${}",
        &setting[3..14],
        salt,
        Base64Bcrypt::encode_string(&output)
    ))
}
