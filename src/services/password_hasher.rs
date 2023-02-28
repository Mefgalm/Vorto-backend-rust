use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2
};
use rand_core::OsRng;

use crate::error::VortoResult::{self, *};

pub struct PwdHasher {
    argon: Argon2<'static>
}

impl PwdHasher {
    pub fn new() -> Self {
        Self {
            argon: Argon2::default()
        }
    }

    pub fn hash_password(&self, password: &str) -> VortoResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(self.argon.hash_password(password.as_bytes(), salt.as_ref())?.to_string())
    }
    
    pub fn verify(&self, password: &str, hashed_password: &str) -> VortoResult<bool> {
        Ok(self.argon.verify_password(password.as_bytes(), &PasswordHash::new(&hashed_password)?).is_ok())
    }
}

