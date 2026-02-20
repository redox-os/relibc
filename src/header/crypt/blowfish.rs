use crate::platform::types::{c_uchar, c_uint};
use alloc::{string::String, vec::Vec};
use base64ct::{Base64Bcrypt, Encoding};
use bcrypt_pbkdf::bcrypt_pbkdf;
use core::str;

const MIN_COST: u32 = 4;
const MAX_COST: u32 = 31;
const BHASH_WORDS: usize = 8;
const BHASH_OUTPUT_SIZE: usize = BHASH_WORDS * 4;

/// Inspired by https://github.com/Keats/rust-bcrypt/blob/87fc59e917bcb6cf3f3752fc7f2b4c659d415597/src/lib.rs#L135
fn split_with_prefix(hash: &str) -> Option<(&str, &str, c_uint)> {
    let valid_prefixes = ["2y", "2b", "2a", "2x"];

    // Should be [prefix, cost, hash]
    let raw_parts: Vec<_> = hash.split('$').skip(1).collect();
    if raw_parts.len() != 3 {
        return None;
    }

    let prefix = raw_parts[0];
    let setting = raw_parts[2];

    if !valid_prefixes.contains(&prefix) {
        return None;
    }

    raw_parts[1]
        .parse::<c_uint>()
        .ok()
        .map(|cost| (prefix, setting, cost))
}

/// Performs Blowfish key derivation on a given password with a specific setting.
///
/// # Parameters
/// * `passw`: The password to be hashed. It must be a string slice (`&str`).
/// * `setting`: The settings for the Blowfish key derivation. It must be a string slice (`&str`)
///   and should follow the format `$<prefix>$<cost>$<setting>`, where `<prefix>` is a string that
///   indicates the type of the hash (e.g., "$2a$"), `<cost>` is a decimal number representing
///   the cost factor for the Blowfish operation, and `<setting>` is a base64-encoded string
///   representing the salt to be used for the Blowfish function.
///
/// # Returns
/// * `Option<String>`: Returns `Some(String)` if the Blowfish operation was successful, where the
///   returned string is the result of the Blowfish operation formatted according to the Modular
///   Crypt Format (MCF). If the Blowfish operation failed, it returns `None`.
///
/// # Errors
/// * If the cost factor is outside the range `[MIN_COST, MAX_COST]`.
///
/// # Example
/// ```
/// let password = "correctbatteryhorsestapler";
/// let setting = "$2y$12$L6Bc/AlTQHyd9liGgGEZyO";
/// let result = crypt_blowfish(password, setting);
/// assert!(result.is_some());
///```
///
/// # Note
/// The `crypt_blowfish` function uses the Blowfish block cipher for hashing.
/// The output of the Blowfish operation is base64-encoded using the BCrypt variant of base64.
pub fn crypt_blowfish(passw: &str, setting: &str) -> Option<String> {
    if let Some((prefix, setting, cost)) = split_with_prefix(setting) {
        if !(MIN_COST..=MAX_COST).contains(&cost) {
            return None;
        }
        // Passwords need to be null terminated
        let mut vec = Vec::with_capacity(passw.len() + 1);
        vec.extend_from_slice(passw.as_bytes());
        vec.push(0);

        // We only consider the first 72 chars; truncate if necessary.
        let passw_t = if vec.len() > 72 { &vec[..72] } else { &vec };
        let passw: &[c_uchar] = passw_t;
        let setting = setting.as_bytes();
        let mut output = vec![0; BHASH_OUTPUT_SIZE + 1];

        bcrypt_pbkdf(passw, setting, cost, &mut output).ok()?;
        Some(format!(
            "${}${}${}",
            prefix,
            cost,
            Base64Bcrypt::encode_string(&output),
        ))
    } else {
        None
    }
}
