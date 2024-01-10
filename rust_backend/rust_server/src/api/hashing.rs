use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, Salt, SaltString,
    },
    Argon2,
};

pub fn check_password(password: &str, password_hash: &str) -> Result<bool, anyhow::Error> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&password_hash).unwrap();
    let result = argon2.verify_password(password.as_bytes(), &parsed_hash);
    match result {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
