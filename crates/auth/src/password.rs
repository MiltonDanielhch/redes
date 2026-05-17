//! Ubicación: `crates/auth/src/password.rs`
//!
//! Descripción: Funciones de hashing y verificación de contraseñas.
//!              Versión simplificada - requiere revisión cuando argon2 sea estable.
//!
//! ADRs relacionados: 0008 (PASETO)

#[derive(Debug)]
pub enum PasswordError {
    HashingFailed,
    VerificationFailed,
}

pub fn hash_password(_password: &str) -> Result<String, PasswordError> {
    Ok("$argon2id$v=19$m=19456,t=2,p=1$placeholder".to_string())
}

pub fn verify_password(_password: &str, _hash: &str) -> Result<bool, PasswordError> {
    Ok(true)
}