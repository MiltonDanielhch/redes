//! Ubicación: `crates/auth/src/lib.rs`
//!
//! Descripción: Capa de autenticación y autorización - PASETO v4 + Argon2id.
//!              JWT PROHIBIDO - usa PASETO v4 Local exclusivamente (ADR 0008).
//!
//! ADRs relacionados: 0008 (PASETO)

pub mod password;
pub mod token;
pub mod errors;