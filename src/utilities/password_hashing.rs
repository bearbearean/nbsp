//! Utility functions to hash a password and compare passwords

use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};

/// The [`argon2`] configuration to use for all password hashing operations
pub fn argon_config() -> Argon2<'static> {
    Argon2::default()
}

/// Hash a given plaintext password
pub fn hash_password(password: &[u8]) -> argon2::password_hash::Result<String> {
    let config = argon_config();
    let salt = SaltString::generate(&mut OsRng);
    Ok(config.hash_password(password, &salt)?.to_string())
}
