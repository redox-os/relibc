use super::gen_salt;
use alloc::string::{String, ToString};
use base64ct::{Base64Bcrypt, Encoding};
use core::str;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

/// Performs PBKDF2 key derivation on a given password with a specific setting.
///
/// # Parameters
/// * `passw`: The password to be hashed. It must be a string slice (`&str`).
/// * `setting`: The settings for the PBKDF2 key derivation. It must be a string slice (`&str`)
///   and should follow the format `$<iter>$<salt>`. The `<iter>` part should be a hexadecimal
///   number representing the iteration count for the PBKDF2 function. The `<salt>` part should
///   be a base64-encoded string representing the salt to be used for the PBKDF2 function.
///
/// # Returns
/// * `Option<String>`: Returns `Some(String)` if the PBKDF2 operation was successful, where the
///   returned string is the result of the PBKDF2 operation formatted according to the Modular
///   Crypt Format (MCF). If the PBKDF2 operation failed, it returns `None`.
///
/// # Errors
/// * If the `setting` does not contain a '$' character.
/// * If the `setting` contains another '$' character after the first one.
/// * If the `<salt>` part of the `setting` is empty.
/// * If the `<iter>` part of the `setting` cannot be converted into a `u32` integer.
///
/// # Example
/// ```
/// let password = "my_password";
/// let setting = "$8$3e8$salt";
/// let result = crypt_pbkdf2(password, setting);
/// assert!(result.is_some());
/// ```
///
/// # Note
/// The `crypt_pbkdf2` function uses the SHA256 hashing algorithm for the PBKDF2 operation.
/// The output of the PBKDF2 operation is base64-encoded using the BCrypt variant of base64.
pub fn crypt_pbkdf2(passw: &str, setting: &str) -> Option<String> {
    if let Some((iter_str, salt)) = &setting[3..].split_once('$') {
        if salt.contains('$') {
            return None;
        }

        let actual_salt = if !salt.is_empty() {
            salt.to_string()
        } else {
            gen_salt()?
        };

        let iter = u32::from_str_radix(iter_str, 16).ok()?;
        let mut buffer = [0u8; 32];
        pbkdf2_hmac::<Sha256>(passw.as_bytes(), actual_salt.as_bytes(), iter, &mut buffer);

        Some(format!(
            "$8${}${}${}",
            iter_str,
            salt,
            Base64Bcrypt::encode_string(&buffer)
        ))
    } else {
        None
    }
}
