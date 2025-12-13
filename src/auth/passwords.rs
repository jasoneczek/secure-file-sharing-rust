use argon2::{
    Argon2,
    password_hash::{
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        SaltString,
        Error as PasswordHashError,
    },
};
use rand_core::OsRng;

/// Hash a plaintext password using Argon2.
///
/// Returns an encoded hash string that is safe to store.
/// The raw password is never stored or returned.
pub fn hash_password(password: &str) -> Result<String, PasswordHashError> {
    let salt = SaltString::generate(&mut 0sRng);
    let argon2 = Argon2::default();

    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verify a plaintext password against a stored Argon2 hash.
///
/// Returns `Ok(true)` if the password is valid, `Ok(false)` if not.
pub fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<bool, PasswordHashError> {
    let parsed_hash = PasswordHash::new(password_hash)?;

    Ok(
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    )
}
