use alloc::string::{String, ToString};

use sha_crypt::{
    sha256_crypt_b64, sha512_crypt_b64, Sha256Params, Sha512Params, ROUNDS_DEFAULT, ROUNDS_MAX,
    ROUNDS_MIN,
};

use crate::platform::types::*;

// key limit is not part of the original design, added for DoS protection.
// rounds limit has been lowered (versus the reference/spec), also for DoS
// protection. runtime is O(klen^2 + klen*rounds)
const KEY_MAX: usize = 256;
const SALT_MAX: usize = 16;
const RSTRING: &str = "rounds=";

pub enum ShaType {
    Sha256,
    Sha512,
}

/// Performs SHA hashing on a given password with a specific setting.
///
/// # Parameters
/// * `passw`: The password to be hashed. It must be a string slice (`&str`).
/// * `setting`: The settings for the SHA hashing. It must be a string slice (`&str`)
///   and should start with "$5$" for SHA256 or "$6$" for SHA512. The rest of the string should represent the salt
///   to be used for the SHA hashing.
/// * `cipher`: The type of SHA algorithm to use. It should be either `ShaType::Sha256` or `ShaType::Sha512`.
///
/// # Returns
/// * `Option<String>`: Returns `Some(String)` if the SHA operation was successful, where the
///   returned string is the result of the SHA operation formatted according to the Modular
///   Crypt Format (MCF). If the SHA operation failed, it returns `None`.
///
/// # Errors
/// * If the `passw` length exceeds `KEY_MAX`.
/// * If the `setting` does not start with "$5$" or "$6$".
/// * If the `setting` does not contain a '$' character.
/// * If the `setting` contains another '$' character after the first one.
/// * If the `setting` contains invalid characters.
/// * If the `setting` contains an invalid number of rounds.
/// * If the `sha256_crypt_b64` or `sha512_crypt_b64` function fails to hash the password.
///
/// # Example
/// ```
/// let password = "my_password";
/// let setting = "$5$rounds=1400$anotherlongsaltstringg";
/// let result = crypt_sha(password, setting, ShaType::Sha256);
/// assert!(result.is_some());
/// ```
///
/// # Note
/// The `crypt_sha` function uses the SHA256 or SHA512 hashing algorithm for hashing.
/// The output of the SHA operation is base64-encoded using the BCrypt variant of base64.
pub fn crypt_sha(passw: &str, setting: &str, cipher: ShaType) -> Option<String> {
    let mut cursor = 3;
    let rounds;

    /* reject large keys */
    if passw.len() > KEY_MAX {
        return None;
    }

    // SHA256
    // setting: $5$rounds=n$setting$ (rounds=n$ and closing $ are optional)
    // SHA512
    // setting: $6$rounds=n$setting$ (rounds=n$ and closing $ are optional)
    let param = match cipher {
        ShaType::Sha256 => "$5$",
        ShaType::Sha512 => "$6$",
    };

    if &setting[0..3] != param {
        return None;
    }

    let has_round;
    // 7 is len("rounds=")
    if &setting[cursor..cursor + 7] == RSTRING {
        cursor += 7;
        has_round = true;
        if let Some(c_end) = setting[cursor..].chars().position(|r| r == '$') {
            if let Ok(u) = setting[cursor..cursor + c_end].parse::<c_ulong>() {
                cursor += c_end + 1;
                rounds = u.min(ROUNDS_MAX as c_ulong).max(ROUNDS_MIN as c_ulong);
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        has_round = false;
        rounds = ROUNDS_DEFAULT as c_ulong;
    }

    let mut slen = cursor;

    for i in 0..SALT_MAX.min(setting.len() - cursor) {
        let idx = cursor + i;

        if &setting[idx..idx + 1] == "$" {
            break;
        }

        // reject characters that interfere with /etc/shadow parsing
        if &setting[idx..idx + 1] == "\n" || &setting[idx..idx + 1] == ":" {
            return None;
        }
        slen += 1;
    }

    let setting = &setting[cursor..slen];

    if let Ok(enc) = match cipher {
        ShaType::Sha256 => {
            let params = Sha256Params::new(rounds as usize)
                .unwrap_or(Sha256Params::new(ROUNDS_DEFAULT).unwrap());
            sha256_crypt_b64(passw.as_bytes(), setting.as_bytes(), &params)
        }
        ShaType::Sha512 => {
            let params = Sha512Params::new(rounds as usize)
                .unwrap_or(Sha512Params::new(ROUNDS_DEFAULT).unwrap());
            sha512_crypt_b64(passw.as_bytes(), setting.as_bytes(), &params)
        }
    } {
        let (r_slice, rn_slice) = if has_round {
            (RSTRING, rounds.to_string() + "$")
        } else {
            ("", String::new())
        };

        Some(format!(
            "{}{}{}{}${}",
            param, r_slice, rn_slice, setting, enc
        ))
    } else {
        None
    }
}
