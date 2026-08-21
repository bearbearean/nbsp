//! Utility functions to hash a password and compare passwords

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
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

/// Verify that a plaintext password matches a hashed password
///
/// Only when this returns `Ok(true)` do the passwords match
pub fn verify_password(
    plaintext_password: &[u8],
    hashed_password: Option<&str>,
) -> argon2::password_hash::Result<bool> {
    if let Some(hashed_password) = hashed_password {
        let config = argon_config();
        let password_hash = PasswordHash::new(hashed_password)?;
        Ok(config
            .verify_password(plaintext_password, &password_hash)
            .is_ok())
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_password_matches() {
        let plaintext = "BEARbearean_123".as_bytes();
        let hash = Some(hash_password(plaintext).unwrap());
        assert!(
            verify_password(plaintext, hash.as_deref()).unwrap(),
            "passwords should match"
        );
    }

    #[test]
    fn test_verify_password_not_matches() {
        let plaintext = "BEARbearean_123".as_bytes();
        let hash = Some(hash_password(b"incorrect").unwrap());
        assert!(
            !verify_password(plaintext, hash.as_deref()).unwrap(),
            "passwords should not match"
        );
    }

    #[test]
    fn test_verify_password_not_matches_none() {
        let plaintext = "BEARbearean_123".as_bytes();
        let hash: Option<String> = None;
        assert!(
            !verify_password(plaintext, hash.as_deref()).unwrap(),
            "verify_password should return false on hashed_password=None"
        );
    }
}
