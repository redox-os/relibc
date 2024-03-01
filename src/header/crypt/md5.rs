use crate::platform::types::c_uchar;
use alloc::string::String;
use base64ct::{Base64ShaCrypt, Encoding};
use core::str;
use md5_crypto::{Digest, Md5};

// Block size for MD5
const BLOCK_SIZE: usize = 16;
// PWD part length of the password string
const PW_SIZE_MD5: usize = 22;
// Maximum length of a setting
const SALT_MAX: usize = 8;
// Inverse encoding map for MD5.
const MAP_MD5: [c_uchar; BLOCK_SIZE] = [12, 6, 0, 13, 7, 1, 14, 8, 2, 15, 9, 3, 5, 10, 4, 11];

const KEY_MAX: usize = 30000;

fn encode_md5(source: &[c_uchar]) -> Option<[c_uchar; PW_SIZE_MD5]> {
    let mut transposed = [0; BLOCK_SIZE];
    for (i, &ti) in MAP_MD5.iter().enumerate() {
        transposed[i] = source[ti as usize];
    }
    let mut buf = [0; PW_SIZE_MD5];
    Base64ShaCrypt::encode(&transposed, &mut buf).ok()?;
    Some(buf)
}

/// Function taken from PR: https://github.com/RustCrypto/password-hashes/pull/351
/// This won't be needed once the PR is merged
fn inner_md5(passw: &str, setting: &str) -> Option<String> {
    let mut digest_b = Md5::default();
    digest_b.update(passw);
    digest_b.update(setting);
    digest_b.update(passw);
    let hash_b = digest_b.finalize();

    let mut digest_a = Md5::default();
    digest_a.update(passw);
    digest_a.update("$1$");
    digest_a.update(setting);

    let mut pw_len = passw.len();
    let rounds = pw_len / BLOCK_SIZE;
    for _ in 0..rounds {
        digest_a.update(hash_b);
    }

    // leftover passw
    digest_a.update(&hash_b[..(pw_len - rounds * BLOCK_SIZE)]);

    while pw_len > 0 {
        match pw_len & 1 {
            0 => digest_a.update(&passw[..1]),
            1 => digest_a.update([0u8]),
            _ => unreachable!(),
        }
        pw_len >>= 1;
    }

    let mut hash_a = digest_a.finalize();

    // Repeatedly run the collected hash value through MD5 to burn
    // CPU cycles
    for i in 0..1000_usize {
        // new hasher
        let mut hasher = Md5::default();

        // Add key or last result
        if (i & 1) != 0 {
            hasher.update(passw);
        } else {
            hasher.update(hash_a);
        }

        // Add setting for numbers not divisible by 3
        if i % 3 != 0 {
            hasher.update(setting);
        }

        // Add key for numbers not divisible by 7
        if i % 7 != 0 {
            hasher.update(passw);
        }

        // Add key or last result
        if (i & 1) != 0 {
            hasher.update(hash_a);
        } else {
            hasher.update(passw);
        }

        // digest_c.clone_from_slice(&hasher.finalize());
        hash_a = hasher.finalize();
    }
    encode_md5(hash_a.as_slice())
        .map(|encstr| format!("$1${}${}", setting, str::from_utf8(&encstr).unwrap()))
}

/// Performs MD5 hashing on a given password with a specific setting.
///
/// # Parameters
/// * `passw`: The password to be hashed. It must be a string slice (`&str`).
/// * `setting`: The settings for the MD5 hashing. It must be a string slice (`&str`)
///   and should start with "$1$". The rest of the string should represent the salt
///   to be used for the MD5 hashing.
///
/// # Returns
/// * `Option<String>`: Returns `Some(String)` if the MD5 operation was successful, where the
///   returned string is the result of the MD5 operation formatted according to the Modular
///   Crypt Format (MCF). If the MD5 operation failed, it returns `None`.
///
/// # Errors
/// * If the `passw` length exceeds `KEY_MAX`.
/// * If the `setting` does not start with "$1$".
///
/// # Example
/// ```
/// let password = "my_password";
/// let setting = "$1$saltstring";
/// let result = crypt_md5(password, setting);
/// assert!(result.is_some());
/// ```
///
/// # Note
/// The `crypt_md5` function uses the MD5 hashing algorithm for hashing.
/// The output of the MD5 operation is base64-encoded using the BCrypt variant of base64.
pub fn crypt_md5(passw: &str, setting: &str) -> Option<String> {
    /* reject large keys */
    if passw.len() > KEY_MAX {
        return None;
    }

    if &setting[0..3] != "$1$" {
        return None;
    }

    let cursor = 3;
    let slen = cursor
        + setting[cursor..cursor + SALT_MAX]
            .chars()
            .take_while(|c| *c != '$')
            .count();
    let setting = &setting[cursor..slen];

    inner_md5(passw, setting)
}
