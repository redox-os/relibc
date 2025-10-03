use alloc::string::{String, ToString};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};

pub fn crypt_argon2(key: &str, setting: &str) -> Option<String> {
    let hash = PasswordHash::new(setting).ok()?;
    let argon2 = Argon2::default();

    if argon2.verify_password(key.as_bytes(), &hash).is_ok() {
        Some(setting.to_string())
    } else {
        None
    }
}
