use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, Salt, SaltString,
    },
    Argon2,
};

pub fn check_password(password: &str, password_hash: &str) -> bool {
    let argon2 = Argon2::default();
    if let Ok(ref parsed_hash) = PasswordHash::new(password_hash) {
        let result = argon2.verify_password(password.as_bytes(), &parsed_hash);
        match result {
            Ok(()) => true,
            Err(_) => false,
        };
    }
    false
}
