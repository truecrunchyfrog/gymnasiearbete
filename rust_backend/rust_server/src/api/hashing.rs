use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::rngs::OsRng;
pub struct HashSalt {
    pub hash: String,
    pub salt: String,
}

pub fn hash_password(pass: &str) -> Result<HashSalt, argon2::password_hash::Error> {
    let password = pass.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt)?.to_string();
    let salt_str = salt.to_string();
    return Ok(HashSalt {
        hash: password_hash,
        salt: salt_str,
    });
}

pub fn check_password(password: String, hash_salt: HashSalt) -> bool {

    let salt = SaltString::from_b64(&hash_salt.salt).expect("Failed to convert to salt");
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();
    return password_hash == hash_salt.hash;
}